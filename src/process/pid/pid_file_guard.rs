use crate::process::pid::pid_error::PidError;
use crate::process::pid::pid_error::PidError::{
    CreatePidFileError, DeletePidFileError, InvalidPidFilePath, OpenPidFileError,
    ParsePidFileContentError, ReadPidFileError, WritePidFileError,
};
use log::{debug, info, warn};
use nix::libc::pid_t;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process;

/// # PID文件守卫
///
/// 用于管理PID文件的生命周期，在对象被销毁时自动清理PID文件
pub struct PidFileGuard {
    pid_file_path: PathBuf,
}

impl Drop for PidFileGuard {
    /// # 自动清理PID文件
    ///
    /// 当PidFileGuard超出作用域时自动调用，删除对应的PID文件
    fn drop(&mut self) {
        let _ = self
            .delete_pid_file_if_my_process()
            .map_err(|e| warn!("Failed to delete PID file: {}", e));
    }
}

impl PidFileGuard {
    /// # 读取PID文件中的进程ID
    ///
    /// 从PID文件中读取保存的进程ID，如果文件不存在或格式错误则返回None
    ///
    /// ## Returns
    ///
    /// 返回Option<i32>，包含读取到的PID或None
    pub fn read_pid(&self) -> Result<Option<pid_t>, PidError> {
        let pid_file_path = &self.pid_file_path;
        debug!("Reading PID from {pid_file_path:?}...");

        // 获取有效的路径
        let path = pid_file_path
            .to_str()
            .ok_or(InvalidPidFilePath(pid_file_path.clone()))?;

        // 检查文件是否存在
        if !pid_file_path.exists() {
            return Ok(None);
        }

        // 安全地打开和读取文件
        let pid_file = File::open(path).map_err(|_| OpenPidFileError(path.to_string()))?;
        let reader = BufReader::new(pid_file);
        let pid = reader
            .lines()
            .next()
            .ok_or(ReadPidFileError(path.to_string()))?
            .map_err(|_| ReadPidFileError(path.to_string()))?
            .trim()
            .parse::<pid_t>()
            .map_err(|_| ParsePidFileContentError(path.to_string()))?;
        Ok(Some(pid))
    }

    /// # 将当前进程ID写入PID文件
    ///
    /// 创建一个PID文件并将当前进程的ID写入其中，同时返回一个PidFileGuard来管理文件的生命周期。
    /// 当PidFileGuard超出作用域时，会自动清理PID文件。
    ///
    /// ## Returns
    ///
    /// 返回 `PidFileGuard` 实例，用于管理PID文件的生命周期
    ///
    /// ## Examples
    ///
    /// ```
    /// let guard = write_pid();
    /// // 当guard超出作用域时，PID文件会被自动删除
    /// ```
    pub fn write_pid(&self) -> Result<(), PidError> {
        let pid_file_path = &self.pid_file_path;
        let pid = process::id();
        debug!("Writing PID {pid} to {pid_file_path:?}...");

        // 获取有效的路径
        let path = pid_file_path
            .to_str()
            .ok_or(InvalidPidFilePath(pid_file_path.clone()))?;

        // 安全地创建和写入PID文件
        let pid_file = File::create(path).map_err(|_| CreatePidFileError(path.to_string()))?;
        let mut writer = BufWriter::new(pid_file);
        writer
            .write_all(pid.to_string().as_bytes())
            .map_err(|_| WritePidFileError(path.to_string()))?;
        Ok(())
    }

    ///
    /// 删除PID文件，如果该文件是由当前进程创建的。
    ///
    /// 该函数首先尝试读取PID文件中的PID。如果文件存在并且其中的PID与当前进程的PID匹配，
    /// 则删除此PID文件。这通常用于确保在进程退出时清理其遗留的PID文件，避免影响后续相同服务的启动。
    ///
    /// # Returns
    ///
    /// - `Ok(())` 如果操作成功完成或没有需要删除的PID文件。
    /// - `Err(Box<dyn std::error::Error>)` 如果在读取或删除PID文件过程中发生错误。
    ///
    /// # Examples
    ///
    /// ```
    /// let result = delete_pid_file_if_my_process();
    /// assert!(result.is_ok());
    /// ```
    /// 注意: 示例假定当前环境允许读写PID文件，并且当前进程确实拥有一个PID文件。
    ///
    fn delete_pid_file_if_my_process(&self) -> Result<(), PidError> {
        // 如果 PID 文件存在且是当前进程创建的，则删除
        if let Ok(Some(pid)) = self.read_pid()
            && pid == process::id() as pid_t
        {
            Self::delete_pid_file(&self.pid_file_path)?;
        }
        Ok(())
    }

    /// # 删除PID文件
    ///
    /// 删除应用程序对应的PID文件。通常在程序正常退出时调用。
    ///
    /// ## Examples
    ///
    /// ```
    /// delete_pid(); // 删除PID文件
    /// ```
    pub(crate) fn delete_pid_file(pid_file_path: &PathBuf) -> Result<(), PidError> {
        info!("Deleting PID file: {pid_file_path:?} ...");
        // 获取有效的路径
        let path = pid_file_path
            .to_str()
            .ok_or(InvalidPidFilePath(pid_file_path.clone()))?;

        std::fs::remove_file(pid_file_path).map_err(|_| DeletePidFileError(path.to_string()))?;
        Ok(())
    }
}
