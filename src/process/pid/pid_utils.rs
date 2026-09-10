//! # PID 工具函数
//!
//! 提供进程ID相关的实用工具函数，主要用于PID文件的操作和管理。
//! 包括PID文件的读取、写入、删除以及进程身份验证等功能。

use crate::process::PidError;
use tracing::{debug, info};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process;

/// # 获取当前进程 ID
///
/// 返回当前进程的操作系统进程 ID（PID）。
///
/// ## 返回值
///
/// 当前进程的 PID，类型为 `u32`。
pub fn get_current_pid() -> u32 {
    process::id()
}

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
/// use wheel_rs::process::get_pid_file_path;
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
/// 从指定路径的PID文件中读取保存的进程ID。
///
/// ## 参数
/// - `pid_file_path`: PID文件的路径。
///
/// ## 返回值
/// - `Ok(Some(pid))`: 成功读取到PID。
/// - `Ok(None)`: 文件不存在。
/// - `Err(PidError)`: 打开、读取或解析文件内容失败。
///
/// ## 错误
/// - `InvalidPidFilePath`: 路径无法转换为字符串。
/// - `OpenPidFile`: 无法打开文件。
/// - `ReadPidFile`: 文件为空或读取失败。
/// - `ParsePidFileContent`: PID内容无法解析为 `u32`。
pub fn read_pid(pid_file_path: &PathBuf) -> Result<Option<u32>, PidError> {
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
        .parse::<u32>()
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
/// ## 错误
/// - `InvalidPidFilePath`: 路径无法转换为字符串。
/// - `CreatePidFile`: 创建文件失败。
/// - `WritePidFile`: 写入文件失败。
pub fn write_pid(pid_file_path: &PathBuf) -> Result<(), PidError> {
    let pid = get_current_pid();
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
/// 删除指定路径的PID文件。注意：若文件不存在，`remove_file` 会返回错误，
/// 该错误会被转换为 `PidError::DeletePidFile`。
///
/// ## 参数
/// - `pid_file_path`: PID文件的路径。
///
/// ## 返回值
/// - `Ok(())`: 成功删除文件。
/// - `Err(PidError)`: 删除文件失败，包括文件不存在或权限不足等情况。
///
/// ## 错误
/// - `InvalidPidFilePath`: 路径无法转换为字符串。
/// - `DeletePidFile`: 删除文件失败，例如文件不存在或权限不足。
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
/// - `Err(PidError)`: PID匹配但删除文件失败。
///
/// ## 注意事项
/// - 读取PID文件时发生的错误会被忽略（视为无需删除），不会传播。
/// - 并发环境下可能存在竞态条件，请确保调用时机安全。
///
/// ## 错误
/// - `DeletePidFile`: 当前进程PID与文件内容匹配，但删除文件失败。
pub fn delete_pid_file_if_my_process(pid_file_path: &PathBuf) -> Result<(), PidError> {
    // 读取PID文件中的PID，并检查是否与当前进程匹配
    if let Ok(Some(pid)) = read_pid(pid_file_path)
        && pid == get_current_pid()
    {
        // 若匹配，则删除PID文件
        delete_pid_file(pid_file_path)?;
    }

    Ok(())
}
