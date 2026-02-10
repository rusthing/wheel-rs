//! # PID 工具函数
//!
//! 提供进程ID相关的实用工具函数，主要用于PID文件的操作和管理。
//! 包括PID文件的读取、写入、删除以及进程身份验证等功能。

use crate::process::PidError;
use libc::pid_t;
use log::{debug, info};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process;

/// # 获取PID文件路径
///
/// 根据应用程序文件路径生成对应的PID文件路径。
/// 通过将原文件的扩展名替换为 `.pid` 来构造PID文件路径。
///
/// ## 参数
///
/// * `app_file_path` - 应用程序文件的路径
///
/// ## 返回值
///
/// 返回构造好的PID文件路径
///
/// ## 示例
///
/// ```
/// use std::path::PathBuf;
/// use wheel_rs::process::pid_utils::get_pid_file_path;
///
/// let app_path = PathBuf::from("/var/run/myapp");
/// let pid_path = get_pid_file_path(&app_path);
/// assert_eq!(pid_path.extension().unwrap(), "pid");
/// ```
pub fn get_pid_file_path(app_file_path: &PathBuf) -> PathBuf {
    let mut pid_file_path = app_file_path.clone();
    pid_file_path.set_extension("pid");
    pid_file_path
}

/// # 读取PID文件中的进程ID
///
/// 从指定路径的PID文件中读取保存的进程ID。如果文件不存在、无法打开或内容格式错误，
/// 则返回 `Ok(None)`。若发生其他I/O错误，则返回相应的 `PidError`。
///
/// ## 参数
/// - `pid_file_path`: PID文件的路径。
///
/// ## 返回值
/// - `Ok(Some(pid))`: 成功读取到PID。
/// - `Ok(None)`: 文件不存在或内容无效。
/// - `Err(PidError)`: 发生I/O错误或其他异常。
///
/// ## 错误类型
/// - `InvalidPidFilePath`: 路径无效。
/// - `OpenPidFileError`: 无法打开文件。
/// - `ReadPidFileError`: 读取文件失败。
/// - `ParsePidFileContentError`: 解析PID内容失败。
pub fn read_pid(pid_file_path: &PathBuf) -> Result<Option<pid_t>, PidError> {
    debug!("Reading PID from {pid_file_path:?}...");

    // 验证路径是否有效
    let path = pid_file_path
        .to_str()
        .ok_or(PidError::InvalidPidFilePath(pid_file_path.clone()))?;

    // 检查文件是否存在
    if !pid_file_path.exists() {
        return Ok(None);
    }

    // 打开文件并读取第一行内容
    let pid_file = File::open(path).map_err(|_| PidError::OpenPidFile(path.to_string()))?;
    let reader = BufReader::new(pid_file);
    let pid = reader
        .lines()
        .next()
        .ok_or(PidError::ReadPidFile(path.to_string()))?
        .map_err(|_| PidError::ReadPidFile(path.to_string()))?
        .trim()
        .parse::<pid_t>()
        .map_err(|_| PidError::ParsePidFileContent(path.to_string()))?;

    Ok(Some(pid))
}

/// # 将当前进程ID写入PID文件
///
/// 创建或覆盖指定路径的PID文件，并将当前进程的ID写入其中。该操作通常用于标识进程的唯一性。
///
/// ## 参数
/// - `pid_file_path`: PID文件的路径。
///
/// ## 返回值
/// - `Ok(())`: 成功写入PID文件。
/// - `Err(PidError)`: 发生I/O错误或其他异常。
///
/// ## 注意事项
/// - 若文件已存在，会被覆盖。
/// - 确保调用者具有足够的文件系统权限。
/// - 并发访问可能导致冲突，请谨慎使用。
///
/// ## 错误类型
/// - `InvalidPidFilePath`: 路径无效。
/// - `CreatePidFileError`: 创建文件失败。
/// - `WritePidFileError`: 写入文件失败。
pub fn write_pid(pid_file_path: &PathBuf) -> Result<(), PidError> {
    let pid = process::id();
    debug!("Writing PID {pid} to {pid_file_path:?}...");

    // 验证路径是否有效
    let path = pid_file_path
        .to_str()
        .ok_or(PidError::InvalidPidFilePath(pid_file_path.clone()))?;

    // 创建文件并写入当前进程ID
    let pid_file = File::create(path).map_err(|_| PidError::CreatePidFile(path.to_string()))?;
    let mut writer = BufWriter::new(pid_file);
    writer
        .write_all(pid.to_string().as_bytes())
        .map_err(|_| PidError::WritePidFile(path.to_string()))?;

    Ok(())
}

/// # 删除PID文件
///
/// 删除指定路径的PID文件。如果文件不存在，则操作被视为成功。
///
/// ## 参数
/// - `pid_file_path`: PID文件的路径。
///
/// ## 返回值
/// - `Ok(())`: 成功删除文件或文件不存在。
/// - `Err(PidError)`: 删除文件失败。
///
/// ## 错误类型
/// - `InvalidPidFilePath`: 路径无效。
/// - `DeletePidFileError`: 删除文件失败。
pub fn delete_pid_file(pid_file_path: &PathBuf) -> Result<(), PidError> {
    info!("Deleting PID file: {pid_file_path:?} ...");

    // 验证路径是否有效
    let path = pid_file_path
        .to_str()
        .ok_or(PidError::InvalidPidFilePath(pid_file_path.clone()))?;

    // 删除文件（若文件不存在则视为成功）
    std::fs::remove_file(pid_file_path).map_err(|_| PidError::DeletePidFile(path.to_string()))?;

    Ok(())
}

/// # 删除PID文件（仅限当前进程创建的文件）
///
/// 检查指定路径的PID文件是否由当前进程创建。如果是，则删除该文件；
/// 否则不执行任何操作。此函数常用于进程退出时的安全清理。
///
/// ## 参数
/// - `pid_file_path`: PID文件的路径。
///
/// ## 返回值
/// - `Ok(())`: 成功完成操作或无需删除。
/// - `Err(PidError)`: 读取或删除文件失败。
///
/// ## 注意事项
/// - 此函数依赖于 `read_pid` 和 `delete_pid_file` 的正确实现。
/// - 并发环境下可能存在竞态条件，请确保调用时机安全。
///
/// ## 错误类型
/// - 继承自 `read_pid` 和 `delete_pid_file` 的错误类型。
pub fn delete_pid_file_if_my_process(pid_file_path: &PathBuf) -> Result<(), PidError> {
    // 读取PID文件中的PID，并检查是否与当前进程匹配
    if let Ok(Some(pid)) = read_pid(pid_file_path)
        && pid == process::id() as pid_t
    {
        // 若匹配，则删除PID文件
        delete_pid_file(pid_file_path)?;
    }

    Ok(())
}
