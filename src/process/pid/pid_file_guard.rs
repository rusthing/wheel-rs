//! # PID 文件守卫模块
//!
//! 本模块提供了 `PidFileGuard` 结构体，用于管理 PID 文件的生命周期。
//! 在对象被销毁时会自动清理对应的 PID 文件，避免残留文件占用资源。

use crate::process::pid::pid_utils::delete_pid_file_if_my_process;
use crate::process::{write_pid, PidError};
use log::warn;
use std::path::PathBuf;

/// # PID 文件守卫
///
/// 用于管理 PID 文件的生命周期，在对象被销毁时自动清理 PID 文件。
/// 通过实现 `Drop` trait，确保在作用域结束时自动执行清理逻辑。
#[derive(Debug)]
pub struct PidFileGuard {
    /// 存储 PID 文件的路径
    pid_file_path: PathBuf,
}

impl Drop for PidFileGuard {
    /// # 自动清理 PID 文件
    ///
    /// 当 `PidFileGuard` 超出作用域时自动调用此方法，尝试删除对应的 PID 文件。
    /// 如果删除失败，会记录警告日志但不会 panic。
    fn drop(&mut self) {
        if let Err(e) = delete_pid_file_if_my_process(&self.pid_file_path) {
            warn!("Failed to delete PID file: {}", e);
        }
    }
}

impl PidFileGuard {
    /// # 创建新的 PID 文件守卫实例
    ///
    /// 构造一个新的 `PidFileGuard` 实例，并创建对应的 PID 文件。
    ///
    /// ## 参数
    /// - `app_file_path`: 应用程序文件的基础路径，用于生成 `.pid` 文件路径。
    ///
    /// ## 返回值
    /// - 成功时返回 `Ok(PidFileGuard)` 实例。
    /// - 失败时返回 `Err(PidError)`，表示无法创建或写入 PID 文件。
    ///
    /// ## 示例
    /// ```rust
    /// use std::path::PathBuf;
    /// use crate::process::pid::PidFileGuard;
    ///
    /// let app_path = PathBuf::from("/tmp/my_app");
    /// let guard = PidFileGuard::new(&app_path);
    /// ```
    pub fn new(pid_file_path: PathBuf) -> Result<Self, PidError> {
        // 写入当前进程的 PID 到文件中
        write_pid(&pid_file_path)?;

        // 返回成功的守卫实例
        Ok(Self { pid_file_path })
    }
}
