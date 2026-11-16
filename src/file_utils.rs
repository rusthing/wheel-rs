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
//! use std::path::Path;
//! use wheel_rs::file_utils::{calc_hash_of_file, get_file_ext};
//!
//! // 获取文件扩展名（无扩展名时返回 None）
//! let ext = get_file_ext("example.TXT").unwrap();
//! assert_eq!(ext, "txt");
//!
//! // 计算文件哈希值（文件不存在时返回 Err）
//! if let Ok(hash) = calc_hash_of_file(Path::new("test.txt")) {
//!     println!("文件哈希值: {}", hash);
//! }
//! ```

use notify::{Event, RecursiveMode, Watcher};
use sha2::Digest;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::{Instant, sleep_until};
use tracing::{error, info, warn};

/// 去抖动定时器「未激活」时的截止时间偏移：一年。
///
/// 这里不能直接用 `Duration::from_millis(u64::MAX)` 之类的极值：`Instant + Duration`
/// 在溢出时按平台不同可能直接 panic，而「永不触发」只需要一个远大于任何
/// 去抖动窗口、又不会越过 `Instant` 可表示范围的偏移。
const IDLE_DEADLINE: Duration = Duration::from_secs(365 * 24 * 60 * 60);

/// # 获取文件名的扩展名
///
/// 语义与 [`std::path::Path::extension`] 一致：扩展名是文件名中最后一个点（`.`）之后的部分，
/// 返回时统一转为小写。
///
/// ## 参数
///
/// * `file_name` - 文件名或路径字符串切片
///
/// ## 返回值
///
/// 返回小写化的扩展名（不含点号）；下列情况返回 `None`：
/// - 没有文件名部分（`""`、`"/"`、`"dir/"`）
/// - 文件名以 `.` 开头且不含其它 `.`（`".gitignore"` 是隐藏文件，不是「扩展名为 gitignore」）
/// - 文件名中不含 `.`（`"README"`）
///
/// 几种边界输入的行为：
///
/// | 输入 | 返回 |
/// | --- | --- |
/// | `"example.TXT"` | `Some("txt")` |
/// | `"a.b.c"` | `Some("c")` |
/// | `"file_without_extension"` | `None` |
/// | `".gitignore"` | `None` |
/// | `"trailing."` | `Some("")` |
/// | `""` | `None` |
///
/// ## 示例
///
/// ```
/// use wheel_rs::file_utils::get_file_ext;
///
/// assert_eq!(get_file_ext("example.TXT").as_deref(), Some("txt"));
/// assert_eq!(get_file_ext("document.pdf").as_deref(), Some("pdf"));
/// assert_eq!(get_file_ext("a.b.c").as_deref(), Some("c"));
/// assert_eq!(get_file_ext("README"), None);
/// // 隐藏文件没有扩展名
/// assert_eq!(get_file_ext(".gitignore"), None);
/// ```
pub fn get_file_ext(file_name: &str) -> Option<String> {
    // 交给 std 判定「什么算扩展名」：它已经处理了隐藏文件、结尾点、路径分隔符等边界
    Path::new(file_name)
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
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
/// * `Ok(String)` - 文件 SHA256 哈希值的小写十六进制字符串（64 个字符）
/// * `Err(io::Error)` - 文件无法打开或读取过程中出错，**不会 panic**
///
/// ## 示例
///
/// ```
/// use std::path::Path;
/// use wheel_rs::file_utils::calc_hash_of_file;
///
/// // 假设存在一个名为 "test.txt" 的文件；文件不存在时返回 Err 而非 panic
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
    #[cfg(unix)]
    {
        // EXDEV 是唯一来源；同时看 ErrorKind 与原始错误码，避免某些封装层
        // 只保留了 raw_os_error 而没映射到 CrossesDevices。
        err.kind() == io::ErrorKind::CrossesDevices || err.raw_os_error() == Some(libc::EXDEV)
    }
    #[cfg(windows)]
    {
        // ERROR_NOT_SAME_DEVICE = 17
        err.raw_os_error() == Some(17)
    }
}

/// # 解析软链，得到事件真正会产生的那个路径
///
/// [`std::fs::canonicalize`] 要求路径上每一级都存在，而这里的目标文件可能暂时不存在
/// （比如容器里 `log.toml -> config/log.toml`，config 目录还没就绪）。所以自己逐级跟随
/// 软链：只看最后一级是不是软链，是就跳过去，最多跳 8 层防止环。
fn resolve_symlink(path: &Path) -> PathBuf {
    let mut cur = path.to_path_buf();
    for _ in 0..8 {
        match std::fs::symlink_metadata(&cur) {
            Ok(md) if md.file_type().is_symlink() => match std::fs::read_link(&cur) {
                Ok(target) => {
                    cur = if target.is_absolute() {
                        target
                    } else {
                        cur.parent().unwrap_or(Path::new(".")).join(target)
                    };
                }
                Err(_) => break,
            },
            _ => break,
        }
    }
    cur
}

/// # 取「最近的已存在祖先目录」并规范化
///
/// 目录尚未创建时（例如配置目录还没挂进来）返回最近的已存在上级，
/// 避免 `watch` 直接失败导致整个监听列表为空。
fn nearest_existing_dir(path: &Path) -> PathBuf {
    let mut dir = path.to_path_buf();
    while !dir.is_dir() {
        match dir.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => dir = parent.to_path_buf(),
            _ => break,
        }
    }
    std::fs::canonicalize(&dir).unwrap_or(dir)
}

/// # 文件监听器
///
/// 封装 `notify` 的底层 watcher 与两个后台 tokio 任务（去抖动任务、事件转发任务）。
/// 通常不需要直接构造，优先使用 [`watch_file_changed`]。
///
/// 持有期间持续监听文件变更；被 drop 时会自动 abort 两个后台任务，
/// 因此**必须保持该值存活**，否则监听会立即停止。
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
    /// # 创建文件监听器
    ///
    /// 内部完成三件事：创建 `notify` watcher、启动去抖动任务、启动事件消费任务
    /// （把去抖动后的事件交给 `on_change`）。只关注修改（modify）、删除（remove）
    /// 与创建（create）事件，其余事件会被直接忽略。
    ///
    /// 实际的 `watch` 落在**目标文件所在目录**（`RecursiveMode::NonRecursive`），
    /// 再用「目录 + 文件名」过滤事件，而不是直接 `watch` 文件：
    /// inotify 对文件的 watch 绑在该文件的 inode 上，文件被 rename / 临时文件+rename
    /// 覆盖后内核会连 watch 一起摘掉，导致热更新只能生效一次。目标路径是软链时，
    /// 会先跟随软链找到真实文件，再监听真实文件所在目录。
    ///
    /// 目标文件当前不存在也不会报错：监听落在最近的已存在祖先目录上，
    /// 文件之后被创建同样能收到事件。
    ///
    /// 需在 tokio 运行时中调用，因为内部使用 `tokio::spawn`。
    ///
    /// 相比把「事件发送端 + 消费任务句柄」交给调用方自己拼装，这里把两者都收在内部，
    /// 避免句柄与通道配错（drop 时 abort 掉别人的任务）。
    ///
    /// ## 参数
    ///
    /// * `files` - 要监听的文件路径列表
    /// * `debounce_delay` - 去抖动延迟；该时间窗内的连续变更只会上报最后一次
    /// * `on_change` - 变更回调；返回的 `Err` 只记 warn 日志，不会中断监听
    ///
    /// ## 返回值
    ///
    /// * `Ok(FileWatcher)` - 监听已启动
    /// * `Err(notify::Error)` - watcher 创建失败或某个路径无法监听
    pub fn new<F, Fut>(
        files: &[String],
        debounce_delay: Duration,
        mut on_change: F,
    ) -> notify::Result<Self>
    where
        F: FnMut(Event) -> Fut + Send + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let (file_changed_tx, mut file_changed_rx) = watch::channel(Event::default());

        // 事件消费任务：把去抖动后的事件交给回调
        let watch_join_handle = tokio::spawn(async move {
            loop {
                match file_changed_rx.changed().await {
                    Ok(_) => {
                        let event = file_changed_rx.borrow().clone();
                        info!(
                            "watch task received event, calling on_change: {:?}",
                            event.paths
                        );
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

        let (event_tx, mut event_rx) = watch::channel(Event::default());

        // 去抖动任务
        let debounce_join_handle = tokio::spawn(async move {
            let sleep = sleep_until(Instant::now() + IDLE_DEADLINE); // 初始为「永不触发」
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
                        // 重新设置为「永不触发」，直到下次监听到变化
                        sleep.as_mut().reset(Instant::now() + IDLE_DEADLINE);
                    }
                }
            }
        });

        // 把「要监听的文件」换算成「要监听的目录 + 目标文件名」。
        //
        // inotify 对**文件**的 watch 绑在该文件的 inode 上：文件被 rename / 临时文件+rename
        // 覆盖（换了 inode）之后，内核会把 watch 一并摘掉（IN_IGNORED），此后再也不产生事件
        // ——热更新只能生效一次。目录的 watch 不会因子文件被替换而失效，因此统一监听父目录，
        // 再按「目录 + 文件名」把同目录下临时文件、备份、日志等杂事件过滤掉。
        let mut targets: Vec<(PathBuf, std::ffi::OsString)> = Vec::new();
        let mut watch_dirs: Vec<PathBuf> = Vec::new();
        for file in files {
            // 目标是软链（容器里 log.toml -> config/log.toml）时要跟到真实文件：
            // 事件产生在真实文件所在的目录，落在软链所在的 app_dir 上是匹配不到的。
            let real = resolve_symlink(Path::new(file));
            let Some(name) = real.file_name().map(|n| n.to_os_string()) else {
                warn!("监听目标没有文件名，忽略: {}", file);
                continue;
            };
            let dir = nearest_existing_dir(real.parent().unwrap_or(Path::new(".")));
            if !watch_dirs.contains(&dir) {
                watch_dirs.push(dir.clone());
            }
            targets.push((dir, name));
        }

        info!("file watcher start watching: {:?}", files);
        info!("file watcher watches dirs: {:?}", watch_dirs);
        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                Ok(event) => {
                    info!(
                        "notify raw event: kind={:?}, paths={:?}",
                        event.kind, event.paths
                    );
                    if !(event.kind.is_modify()
                        || event.kind.is_remove()
                        || event.kind.is_create())
                    {
                        return;
                    }
                    // 只认「目标文件所在目录 + 目标文件名」，其它子文件（临时文件、.bak、日志）放行不走
                    let hit = event.paths.iter().any(|p| {
                        targets.iter().any(|(dir, target_name)| {
                            p.parent() == Some(dir.as_path())
                                && p.file_name() == Some(target_name.as_os_str())
                        })
                    });
                    if !hit || event.paths.is_empty() {
                        return;
                    }
                    let _ = event_tx.send(event);
                }
                Err(e) => {
                    error!("notify watcher error: {:?}", e);
                }
            })?;

        for dir in &watch_dirs {
            info!("watching dir: {}", dir.display());
            watcher.watch(dir, RecursiveMode::NonRecursive)?;
        }

        Ok(Self {
            _watcher: Box::new(watcher),
            debounce_join_handle,
            watch_join_handle,
        })
    }
}

/// # 监听文件变更并执行回调（推荐入口）
///
/// 对给定文件启动监听，变更事件经 `debounce_delay` 去抖动后触发 `on_change` 回调。
/// 行为与 [`FileWatcher::new`] 完全一致，区别只在于 `files` 收 `Vec<String>`（可直接传拥有所有权的列表）。
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
    on_change: F,
) -> notify::Result<FileWatcher>
where
    F: FnMut(Event) -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    FileWatcher::new(&files, debounce_delay, on_change)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_ext_matches_documented_table() {
        assert_eq!(get_file_ext("example.TXT").as_deref(), Some("txt"));
        assert_eq!(get_file_ext("document.pdf").as_deref(), Some("pdf"));
        assert_eq!(get_file_ext("a.b.c").as_deref(), Some("c"));
        assert_eq!(get_file_ext("file_without_extension"), None);
        // 隐藏文件按 std 语义没有扩展名
        assert_eq!(get_file_ext(".gitignore"), None);
        assert_eq!(get_file_ext("trailing.").as_deref(), Some(""));
        assert_eq!(get_file_ext(""), None);
        // 路径形式：只看最后一段
        assert_eq!(get_file_ext("/tmp/dir/photo.JPG").as_deref(), Some("jpg"));
        assert_eq!(get_file_ext("dir/"), None);
    }

    #[test]
    fn cross_device_error_detection() {
        #[cfg(unix)]
        {
            let exdev = io::Error::from_raw_os_error(libc::EXDEV);
            assert!(is_cross_device_error(&exdev));
        }
        let other = io::Error::new(io::ErrorKind::InvalidInput, "bad input");
        assert!(!is_cross_device_error(&other));
    }

    #[test]
    fn resolve_symlink_and_nearest_dir() {
        let root = std::env::temp_dir().join("wheel_rs_link_test");
        std::fs::create_dir_all(root.join("config")).unwrap();
        let real = root.join("config/log.toml");
        std::fs::write(&real, b"level = \"info\"").unwrap();

        // 普通文件原样返回
        assert_eq!(resolve_symlink(&real), real);

        #[cfg(unix)]
        {
            // 软链解析到真实文件：监听必须落在真实文件所在的 config 目录上，
            // 否则 rename 覆盖的事件在软链所在的 app_dir 下匹配不到。
            let link = root.join("log.toml");
            std::fs::remove_file(&link).ok();
            std::os::unix::fs::symlink("config/log.toml", &link).unwrap();
            assert_eq!(resolve_symlink(&link), real);
        }

        // 不存在的路径原样返回，不 panic
        let missing = root.join("nope.toml");
        assert_eq!(resolve_symlink(&missing), missing);

        // 目录还没建时落到最近的已存在祖先
        let deep = root.join("a/b/c");
        assert_eq!(
            nearest_existing_dir(&deep),
            std::fs::canonicalize(&root).unwrap()
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn hash_of_temp_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("wheel_rs_hash_test.txt");
        std::fs::write(&path, b"hello").unwrap();
        let hash = calc_hash_of_file(&path).unwrap();
        // sha256("hello")
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
        std::fs::remove_file(&path).ok();
    }
}
