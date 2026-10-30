//! # PID 工具函数
//!
//! 提供进程ID相关的实用工具函数，主要用于PID文件的操作和管理。
//! 包括PID文件的读取、写入、删除以及进程身份验证等功能。

use crate::process::PidError;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process;
use tracing::{debug, info};

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
pub fn get_pid_file_path(app_file_path: &Path) -> PathBuf {
    let mut pid_file_path = app_file_path.to_path_buf();
    pid_file_path.set_extension("pid");
    pid_file_path
}

/// # 读取PID文件中的进程ID
///
/// 从指定路径的PID文件中读取保存的进程ID（只读第一行，前后空白会被忽略）。
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
pub fn read_pid(pid_file_path: &Path) -> Result<Option<u32>, PidError> {
    debug!("Reading PID from {pid_file_path:?}...");

    // 路径需要能转成字符串，否则后续错误信息无法给出可读路径
    let path = pid_file_path
        .to_str()
        .ok_or_else(|| PidError::InvalidPidFilePath(pid_file_path.to_path_buf()))?;

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
/// - `WritePidFile`: 写入或刷新文件失败。
pub fn write_pid(pid_file_path: &Path) -> Result<(), PidError> {
    let pid = get_current_pid();
    debug!("Writing PID {pid} to {pid_file_path:?}...");

    // 路径需要能转成字符串，否则后续错误信息无法给出可读路径
    let path = pid_file_path
        .to_str()
        .ok_or_else(|| PidError::InvalidPidFilePath(pid_file_path.to_path_buf()))?;

    // 创建文件并写入当前进程ID
    let pid_file = File::create(path).map_err(|_| PidError::CreatePidFile(path.to_string()))?;
    let mut writer = BufWriter::new(pid_file);
    writer
        .write_all(pid.to_string().as_bytes())
        .map_err(|_| PidError::WritePidFile(path.to_string()))?;
    // 必须显式 flush：BufWriter 在 drop 时虽然会 flush，但错误会被丢弃，
    // 那样磁盘写满等故障就会表现为「写 PID 成功，文件却是空的」。
    writer
        .flush()
        .map_err(|_| PidError::WritePidFile(path.to_string()))?;

    Ok(())
}

/// # 删除PID文件
///
/// 删除指定路径的PID文件。注意：若文件不存在，`remove_file` 会返回错误，
/// 该错误会被转换为 `PidError::DeletePidFile`；只想在「文件属于本进程」时
/// 清理，请用 [`delete_pid_file_if_my_process`]。
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
pub fn delete_pid_file(pid_file_path: &Path) -> Result<(), PidError> {
    info!("Deleting PID file: {pid_file_path:?} ...");

    // 路径需要能转成字符串，否则后续错误信息无法给出可读路径
    let path = pid_file_path
        .to_str()
        .ok_or_else(|| PidError::InvalidPidFilePath(pid_file_path.to_path_buf()))?;

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
pub fn delete_pid_file_if_my_process(pid_file_path: &Path) -> Result<(), PidError> {
    // 读取PID文件中的PID，并检查是否与当前进程匹配
    if let Ok(Some(pid)) = read_pid(pid_file_path)
        && pid == get_current_pid()
    {
        // 若匹配，则删除PID文件
        delete_pid_file(pid_file_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_pid_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("wheel_rs_{name}_{}.pid", std::process::id()))
    }

    #[test]
    fn write_read_delete_round_trip() {
        let path = temp_pid_path("round_trip");
        let _ = std::fs::remove_file(&path);

        // 文件不存在时读取返回 None
        assert_eq!(read_pid(&path).unwrap(), None);

        write_pid(&path).unwrap();
        assert_eq!(read_pid(&path).unwrap(), Some(get_current_pid()));

        delete_pid_file(&path).unwrap();
        assert_eq!(read_pid(&path).unwrap(), None);
    }

    #[test]
    fn delete_if_my_process_only_removes_own_file() {
        let path = temp_pid_path("guard");
        let _ = std::fs::remove_file(&path);

        // 写入别人的 PID，不应被删除
        std::fs::write(&path, b"1").unwrap();
        delete_pid_file_if_my_process(&path).unwrap();
        assert!(path.exists(), "非本进程的 PID 文件不应被删除");

        // 写入自己的 PID，应被删除
        write_pid(&path).unwrap();
        delete_pid_file_if_my_process(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn pid_file_path_swaps_extension() {
        assert_eq!(
            get_pid_file_path(Path::new("/var/run/myapp")),
            PathBuf::from("/var/run/myapp.pid")
        );
        assert_eq!(
            get_pid_file_path(Path::new("/var/run/myapp.conf")),
            PathBuf::from("/var/run/myapp.pid")
        );
    }

    #[test]
    fn parse_error_is_reported() {
        let path = temp_pid_path("bad_content");
        std::fs::write(&path, b"not-a-pid").unwrap();
        assert!(matches!(
            read_pid(&path),
            Err(PidError::ParsePidFileContent(_))
        ));
        std::fs::remove_file(&path).ok();
    }
}
