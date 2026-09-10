//! # 文件工具模块
//!
//! 提供文件操作相关的实用工具函数：
//! - 获取文件扩展名
//! - 计算文件的 SHA256 哈希值
//! - 检测跨设备操作错误
//! - 监听文件变更（带去抖动）
//!
//! ## 示例
//!
//! ```
//! use wheel_rs::file_utils::get_file_ext;
//!
//! assert_eq!(get_file_ext("example.TXT").as_deref(), Some("txt"));
//! // 获取文件扩展名
//! let ext = get_file_ext("example.TXT").unwrap();
//! assert_eq!(ext, "txt");
//!
//! // 计算文件哈希值
//! if let Ok(hash) = calc_hash_of_file(Path::new("test.txt")) {
//!     println!("文件哈希值: {}", hash);
//! }
//! ```

use notify::{Event, RecursiveMode, Watcher};
use sha2::Digest;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::{sleep_until, Instant};
use tracing::{error, info, warn};

/// # 获取文件名的扩展名
///
/// 该函数从给定的文件名中提取扩展名部分。扩展名被定义为文件名中最后一个点（`.`）之后的部分，
/// 并且会被转换为小写形式。
///
/// ## 参数
///
/// * `file_name` - 包含文件名的字符串切片引用
///
/// ## 返回值
///
/// 返回小写化的扩展名（不含点号）。**该函数始终返回 `Some`**，`Option` 只是历史签名遗留：
/// 输入不含点号时，`split('.').last()` 得到的是整个输入本身，而不是空字符串。
/// 需要区分"无扩展名"时，调用方应自行判断输入是否包含 `.`。
///
/// 几种边界输入的实测行为：
///
/// | 输入 | 返回 |
/// | --- | --- |
/// | `"example.TXT"` | `Some("txt")` |
/// | `"file_without_extension"` | `Some("file_without_extension")` |
/// | `".gitignore"` | `Some("gitignore")` |
/// | `"trailing."` | `Some("")` |
/// | `""` | `Some("")` |
///
/// ## 示例
///
/// ```
/// use wheel_rs::file_utils::get_file_ext;
///
/// assert_eq!(get_file_ext("example.TXT").as_deref(), Some("txt"));
/// assert_eq!(get_file_ext("document.pdf").as_deref(), Some("pdf"));
/// assert_eq!(get_file_ext("a.b.c").as_deref(), Some("c"));
/// // 注意：无扩展名时返回的是整个文件名，而非空字符串
/// assert_eq!(get_file_ext("README").as_deref(), Some("readme"));
/// ```
pub fn get_file_ext(file_name: &str) -> Option<String> {
    file_name
        .split('.')
        .last()
        .map(|s| s.to_string().to_lowercase())
}

/// # 计算指定文件的 SHA256 哈希值
///
/// 打开指定路径的文件并计算其完整 SHA256 哈希值，内部使用 8192 字节缓冲区流式读取，
/// 因此可以处理远大于内存的文件。
///
/// ## 参数
///
/// * `path` - 要计算哈希值的文件路径
///
/// ## 返回值
///
/// * `Ok(String)` - 文件 SHA256 哈希值的小写十六进制字符串（64 个字符）。
/// * `Err(io::Error)` - 文件无法打开或读取过程中出错，**不会 panic**。
/// 返回表示文件 SHA256 哈希值的小写十六进制字符串。
///
/// ## 错误
///
/// 当无法打开文件或读取过程中发生错误时返回 [`io::Error`]。
///
/// ## 示例
///
/// ```rust
/// use std::path::Path;
/// use wheel_rs::file_utils::calc_hash_of_file;
///
/// // 需要替换为真实存在的文件路径；文件不存在时返回 Err 而非 panic
/// let hash = calc_hash_of_file(Path::new("test.txt"))?;
/// println!("文件哈希值: {hash}");
/// # Ok::<(), std::io::Error>(())
/// // 假设存在一个名为 "test.txt" 的文件
/// if let Ok(hash) = calc_hash_of_file(Path::new("test.txt")) {
///     println!("文件哈希值: {}", hash);
/// }
/// ```
pub fn calc_hash_of_file(path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = sha2::Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// # 检查 IO 错误是否为跨设备错误
///
/// 跨设备错误通常发生在尝试移动或重命名文件时，源文件和目标路径位于不同的文件系统或设备上。
/// 此函数检测不同操作系统上的跨设备错误：
/// - 在 Unix 系统上检查 EXDEV 错误 (错误码 18)
/// - 在 Windows 系统上检查 ERROR_NOT_SAME_DEVICE 错误 (错误码 17)
///
/// ## 参数
///
/// * `err` - 待检查的 IO 错误引用
///
/// ## 返回值
///
/// 是跨设备错误则返回 `true`，否则返回 `false`。
///
/// ## 示例
///
/// ```
/// use std::io;
/// use wheel_rs::file_utils::is_cross_device_error;
///
/// #[cfg(unix)]
/// {
///     let exdev = io::Error::from(io::ErrorKind::CrossesDevices);
///     assert!(is_cross_device_error(&exdev));
/// }
///
/// // 其他类型的错误不会被判定为跨设备错误
/// let invalid = io::Error::new(io::ErrorKind::InvalidInput, "bad input");
/// assert!(!is_cross_device_error(&invalid));
/// ```
pub fn is_cross_device_error(err: &io::Error) -> bool {
    match err.kind() {
        // 在 Unix 系统上，跨设备错误通常表现为 CrossesDevices
        #[cfg(unix)]
        io::ErrorKind::CrossesDevices => true,
        #[cfg(unix)]
        _ => false,
        // 在 Windows 系统上，跨设备错误可能表现为 Other 或其他类型
        #[cfg(windows)]
        _ => {
            // Windows 上的跨设备错误通常包含特定的错误信息
            if let Some(raw_os_error) = err.raw_os_error() {
                raw_os_error == 17 // ERROR_NOT_SAME_DEVICE 错误码
            } else {
                false
            }
        }
    }
}

/// # 文件监听器
///
/// 封装 `notify` 的底层 watcher 与两个后台 tokio 任务（去抖动任务、事件转发任务）。
/// 通常不需要直接构造，优先使用 [`watch_file_changed`]。
///
/// 持有期间持续监听文件变更；被 drop 时会自动 abort 两个后台任务，
/// 因此**必须保持该值存活**，否则监听会立即停止。
/// # 文件监视器
///
/// 监视一组文件的修改/删除事件，并在去抖时间窗口后转发最新事件。
///
/// 内部包含：
/// - 底层文件监视器（`notify`）：负责收集文件系统事件
/// - 去抖任务：事件发生后等待 `debounce_delay`，期间不断刷新，最终只转发最新事件
/// - 回调任务（由 [`watch_file_changed`] 创建）：负责消费去抖后的事件并执行用户回调
///
/// `Drop` 时会中止内部去抖任务与回调任务。
pub struct FileWatcher {
    _watcher: Box<dyn Watcher>,
    debounce_join_handle: tokio::task::JoinHandle<()>,
    watch_join_handle: tokio::task::JoinHandle<()>,
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        self.debounce_join_handle.abort();
        self.watch_join_handle.abort();
    }
}

impl FileWatcher {
    /// # 创建文件监视器
    ///
    /// 创建底层文件监视器并启动去抖任务。当任一被监视文件发生修改或删除事件时，
    /// 事件会在 `debounce_delay` 去抖窗口结束后通过 `file_changed_tx` 发送。
    ///
    /// ## 参数
    ///
    /// * `files` - 要监视的文件路径列表
    /// * `debounce_delay` - 去抖延迟时长，事件停止发生该时长后才转发最新事件
    /// * `file_changed_tx` - 用于发送去抖后事件（[`notify::Event`]）的 `watch` 通道发送者
    /// * `watch_join_handle` - 消费 `file_changed_tx` 事件的用户回调任务句柄
    ///
    /// ## 返回值
    ///
    /// 成功返回 [`FileWatcher`] 实例；创建监视器或注册文件失败时返回 [`notify::Error`]。
    /// # 创建文件监听器
    ///
    /// 启动去抖动任务并开始监听给定文件。只关注修改（modify）与删除（remove）事件，
    /// 其余事件会被直接忽略。监听为非递归模式（`RecursiveMode::NonRecursive`）。
    ///
    /// 需在 tokio 运行时中调用，因为内部使用 `tokio::spawn`。
    ///
    /// ## 参数
    ///
    /// * `files` - 要监听的文件路径列表
    /// * `debounce_delay` - 去抖动延迟；该时间窗内的连续变更只会上报最后一次
    /// * `file_changed_tx` - 去抖动后事件的发送端，由调用方负责消费
    /// * `watch_join_handle` - 消费事件的任务句柄，drop 时会被 abort
    ///
    /// ## 返回值
    ///
    /// * `Ok(FileWatcher)` - 监听已启动
    /// * `Err(notify::Error)` - watcher 创建失败或某个路径无法监听
    pub fn new(
        files: &Vec<String>,
        debounce_delay: Duration,
        file_changed_tx: watch::Sender<Event>,
        watch_join_handle: tokio::task::JoinHandle<()>,
    ) -> notify::Result<Self> {
        let (event_tx, mut event_rx) = watch::channel(Event::default());

        // 去抖动任务
        let debounce_join_handle = tokio::spawn(async move {
            let sleep = sleep_until(Instant::now() + Duration::from_millis(u64::MAX)); // 初始设置为永不触发
            let mut latest_event = Event::default(); // 存储最新事件
            // 把局部变量放到栈上的固定位置，否则在select!中无法保证其内存位置不变
            tokio::pin!(sleep);
            loop {
                tokio::select! {
                    // 当 watch 通道变化时，重置定时器
                    res = event_rx.changed() => {
                        match res {
                            Ok(_) => {
                                // 取出最新事件
                                latest_event = event_rx.borrow().clone();
                                // 重置定时器为 debounce_duration
                                sleep.as_mut().reset(Instant::now() + debounce_delay);
                            }
                            Err(err) => {
                                info!("watch file error: {:?}", err);
                                break;
                            }
                        }
                    }
                    // 定时器到期 -> 输出最新值
                    _ = &mut sleep => {
                        info!("debounce fired, forwarding event: {:?}", latest_event.paths);
                        let _ = file_changed_tx.send(latest_event.clone());
                        // 重新设置为永不触发，直到下次监听到变化
                        sleep.as_mut().reset(Instant::now() + Duration::from_millis(u64::MAX));
                    }
                }
            }
        });

        // 创建 watcher，过滤非修改和删除事件，发送最新事件到 event_tx
        info!("file watcher start watching: {:?}", files);
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    info!("notify raw event: kind={:?}, paths={:?}", event.kind, event.paths);
                    if !(event.kind.is_modify() || event.kind.is_remove()) {
                        return;
                    }
                    if event.clone().paths.into_iter().next().is_some() {
                        let _ = event_tx.send(event);
                    }
                }
                Err(e) => {
                    error!("notify watcher error: {:?}", e);
                }
            }
        })?;

        for file in files {
            info!("watching file: {}", file);
            watcher.watch(Path::new(file), RecursiveMode::NonRecursive)?;
        }

        Ok(Self {
            _watcher: Box::new(watcher),
            debounce_join_handle,
            watch_join_handle,
        })
    }
}

/// # 监视文件变化并执行回调
///
/// 监视给定文件列表的修改/删除事件，去抖后调用 `on_change` 回调。
///
/// ## 参数
///
/// * `files` - 要监视的文件路径列表
/// * `debounce_delay` - 去抖延迟时长，事件停止发生该时长后才触发回调
/// * `on_change` - 收到变化事件后执行的异步回调；回调返回 `anyhow::Result<()>`，
///   执行失败仅记录警告日志，不影响后续监视
///
/// ## 返回值
///
/// 成功返回 [`FileWatcher`]；创建监视器或注册文件失败时返回 [`notify::Error`]。
/// # 监听文件变更并执行回调（推荐入口）
///
/// 对给定文件启动监听，变更事件经 `debounce_delay` 去抖动后触发 `on_change` 回调。
/// 相比直接使用 [`FileWatcher::new`]，此函数已内置事件消费任务，调用方只需提供回调。
///
/// 需在 tokio 运行时中调用。回调返回的 `Err` 不会中断监听，仅记录 warn 日志后继续。
///
/// ## 参数
///
/// * `files` - 要监听的文件路径列表
/// * `debounce_delay` - 去抖动延迟，短时间内的连续变更合并为一次回调
/// * `on_change` - 变更回调，接收去抖动后的 `notify::Event`，返回 `Future`
///
/// ## 返回值
///
/// * `Ok(FileWatcher)` - 监听已启动。**需持有该返回值**，一旦 drop 监听即停止。
/// * `Err(notify::Error)` - watcher 创建或路径监听失败
///
/// ## 示例
///
/// ```no_run
/// use std::time::Duration;
/// use wheel_rs::file_utils::watch_file_changed;
///
/// # async fn run() -> notify::Result<()> {
/// let _watcher = watch_file_changed(
///     vec!["config.toml".to_string()],
///     Duration::from_millis(500),
///     |event| async move {
///         println!("文件已变更: {:?}", event.paths);
///         Ok(())
///     },
/// )?;
/// // _watcher 必须保持存活，否则监听会立即停止
/// # Ok(())
/// # }
/// ```
pub fn watch_file_changed<F, Fut>(
    files: Vec<String>,
    debounce_delay: Duration,
    mut on_change: F,
) -> notify::Result<FileWatcher>
where
    F: FnMut(Event) -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    let (file_changed_tx, mut file_changed_rx) = watch::channel(Event::default());
    let files_clone = files.clone();
    let watch_join_handle = tokio::spawn(async move {
        info!("watch file changed: {:?}", files_clone);
        loop {
            match file_changed_rx.changed().await {
                Ok(_) => {
                    let event = file_changed_rx.borrow().clone();
                    info!("watch task received event, calling on_change: {:?}", event.paths);
                    if let Err(e) = on_change(event).await {
                        warn!("handle file change error: {e:?}");
                    }
                }
                Err(err) => {
                    error!("watch file error: {:?}", err);
                    break;
                }
            }
        }
    });
    let file_watcher = FileWatcher::new(
        files.as_ref(),
        debounce_delay,
        file_changed_tx.clone(),
        watch_join_handle,
    );
    file_watcher
}