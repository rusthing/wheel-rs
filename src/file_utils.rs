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
//! use std::path::Path;
//! use your_crate::utils::file_utils::{get_file_ext, calc_hash};
//!
//! // 获取文件扩展名
//! let ext = get_file_ext("example.TXT");
//! assert_eq!(ext, "txt");
//!
//! // 计算文件哈希值
//! // let hash = calc_hash(Path::new("test.txt"));
//! // println!("文件哈希值: {}", hash);
//! ```
use sha2::Digest;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

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
/// use your_crate::utils::file_utils::get_file_ext;
///
/// assert_eq!(get_file_ext("example.TXT"), "txt");
/// assert_eq!(get_file_ext("document.pdf"), "pdf");
/// assert_eq!(get_file_ext("file_without_extension"), "");
/// ```
pub fn get_file_ext(file_name: &str) -> String {
    if file_name.contains('.') {
        file_name
            .split('.')
            .last()
            .unwrap()
            .to_string()
            .to_lowercase()
    } else {
        String::new()
    }
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
/// use your_crate::utils::file_utils::calc_hash;
///
/// // 假设存在一个名为 "test.txt" 的文件
/// let hash = calc_hash(Path::new("test.txt"));
/// println!("文件哈希值: {}", hash);
/// ```
pub fn calc_hash(path: &Path) -> String {
    let mut file = File::open(path).unwrap();
    let mut hasher = sha2::Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let bytes_read = file.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    format!("{:x}", hasher.finalize())
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
/// use your_crate::utils::file_utils::is_cross_device_error;
///
/// let error = io::Error::new(io::ErrorKind::InvalidInput, "cross-device link");
/// if is_cross_device_error(&error) {
///     println!("检测到跨设备错误");
/// }
/// ```
pub fn is_cross_device_error(err: &io::Error) -> bool {
    match err.kind() {
        // 在 Unix 系统上，跨设备错误通常表现为 InvalidInput
        #[cfg(unix)]
        io::ErrorKind::InvalidInput => {
            // 进一步检查错误码是否为 EXDEV (18)
            if let Some(raw_os_error) = err.raw_os_error() {
                raw_os_error == 18 // EXDEV 错误码
            } else {
                false
            }
        }
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
        _ => false,
    }
}
