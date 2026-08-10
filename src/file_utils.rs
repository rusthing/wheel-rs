//! # 文件工具模块
//! 提供文件操作相关的实用工具函数
//!
//! 该模块包含以下主要功能：
//! - 获取文件扩展名
//! - 计算文件的 SHA256 哈希值
//! - 检测跨设备操作错误
//!
//! ## 示例
//!
//! ```
//! use wheel_rs::file_utils::{get_file_ext, calc_hash_of_file};
//!
//! // 获取文件扩展名
//! let ext = get_file_ext("example.TXT");
//! assert_eq!(ext, "txt");
//!
//! // 计算文件哈希值
//! // let hash = calc_hash(Path::new("test.txt"));
//! // println!("文件哈希值: {}", hash);
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
use tracing::{error, info};

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
/// 返回文件的扩展名（不包括点号），如果文件名中没有点号则返回空字符串。
/// 扩展名会被自动转换为小写形式。
///
/// ## 示例
///
/// ```
/// use wheel_rs::file_utils::get_file_ext;
///
/// assert_eq!(get_file_ext("example.TXT"), "txt");
/// assert_eq!(get_file_ext("document.pdf"), "pdf");
/// assert_eq!(get_file_ext("file_without_extension"), "");
/// ```
pub fn get_file_ext(file_name: &str) -> Option<String> {
    file_name
        .split('.')
        .last()
        .map(|s| s.to_string().to_lowercase())
}

/// # 计算指定文件的 SHA256 哈希值
///
/// 该函数会打开指定路径的文件，并计算其完整的 SHA256 哈希值。
/// 使用 8192 字节的缓冲区以高效地处理大文件。
///
/// ## 参数
///
/// * `path` - 指向要计算哈希值的文件路径
///
/// ## 返回值
///
/// 返回表示文件 SHA256 哈希值的小写十六进制字符串。
///
/// ## Panics
///
/// 当无法打开文件或读取过程中发生错误时，函数会 panic。
/// 在生产环境中应适当处理这些错误情况。
///
/// ## 示例
///
/// ```
/// use std::path::Path;
/// use wheel_rs::file_utils::calc_hash_of_file;
///
/// // 假设存在一个名为 "test.txt" 的文件
/// let hash = calc_hash_of_file(Path::new("test.txt"));
/// println!("文件哈希值: {}", hash);
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
/// * `err` - 要检查的 IO 错误引用
///
/// ## 返回值
///
/// 如果错误是跨设备错误则返回 `true`，否则返回 `false`。
///
/// ## 示例
///
/// ```
/// use std::io;
/// use wheel_rs::file_utils::is_cross_device_error;
///
/// let error = io::Error::new(io::ErrorKind::InvalidInput, "cross-device link");
/// if is_cross_device_error(&error) {
///     println!("检测到跨设备错误");
/// }
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
                        let _ = file_changed_tx.send(latest_event.clone());
                        // 重新设置为永不触发，直到下次监听到变化
                        sleep.as_mut().reset(Instant::now() + Duration::from_millis(u64::MAX));
                    }
                }
            }
        });

        // 创建 watcher，过滤非修改和删除事件，发送最新事件到 event_tx
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                if !(event.kind.is_modify() || event.kind.is_remove()) {
                    return;
                }
                if event.clone().paths.into_iter().next().is_some() {
                    let _ = event_tx.send(event);
                }
            }
        })?;

        for file in files {
            watcher.watch(Path::new(file), RecursiveMode::NonRecursive)?;
        }

        Ok(Self {
            _watcher: Box::new(watcher),
            debounce_join_handle,
            watch_join_handle,
        })
    }
}

pub fn watch_file<F, Fut>(
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
        info!("watch file: {:?}", files_clone);
        loop {
            match file_changed_rx.changed().await {
                Ok(_) => {
                    let event = file_changed_rx.borrow().clone();
                    if let Err(e) = on_change(event).await {
                        error!("handle file change error: {e:?}");
                        break;
                    }
                }
                Err(err) => {
                    info!("watch file error: {:?}", err);
                    break;
                }
            }
        }
        info!("file watcher task exit: {:?}", files_clone);
    });
    let file_watcher = FileWatcher::new(
        files.as_ref(),
        debounce_delay,
        file_changed_tx.clone(),
        watch_join_handle,
    );
    file_watcher
}
