//! # 进程管理工具函数
//!
//! 提供进程终止、状态检查等核心功能的实用工具函数。
//! 该模块封装了底层系统调用，简化了进程管理操作，适用于需要监控或控制外部进程的应用场景。

use crate::process::{send_signal_by_instruction, ProcessError};
use std::io;
use std::time::Duration;
use tokio::time::timeout;

/// # 终止进程
///
/// 发送终止信号给指定进程并等待其退出。该函数是异步的，需在 `tokio` 运行时环境中调用。
///
/// ## 参数
///
/// * `pid` - 目标进程ID。
/// * `wait_timeout` - 等待超时时间，超过该时间将返回错误。
/// * `retry_interval` - 重试间隔时间，用于控制检查频率。
///
/// ## 返回值
///
/// * `Ok(())` - 进程成功终止。
/// * `Err(ProcessError)` - 进程终止失败或等待超时。
///
/// ## 错误处理
///
/// 如果进程在 `wait_timeout` 时间内未退出，将返回 [ProcessExitWaitTimeout] 错误。
///
/// ## 示例
/// ```rust
/// use std::time::Duration;
/// use crate::process::terminate_process;
///
/// #[tokio::main]
/// async fn main() {
/// let result = terminate_process(1234, Duration::from_secs(10), Duration::from_secs(1)).await;
/// assert!(result.is_ok());
/// }
/// ```
pub async fn terminate_process(
    pid: i32,
    wait_timeout: Duration,
    retry_interval: Duration,
) -> Result<(), ProcessError> {
    send_signal_by_instruction("terminate", pid).expect("Failed to send signal: ");
    wait_for_process_exit(pid, wait_timeout, retry_interval).await?;
    Ok(())
}

/// # 等待进程退出
///
/// 循环检查指定进程是否存在，直到进程退出或等待超时。该函数是异步的，内部使用 `tokio::time::sleep` 实现延迟检查。
///
/// ## 参数
///
/// * `pid` - 目标进程ID。
/// * `wait_timeout` - 等待超时时间，超过该时间将返回错误。
/// * `retry_interval` - 重试间隔时间，用于控制检查频率。
///
/// ## 返回值
///
/// * `Ok(())` - 进程成功退出。
/// * `Err(ProcessExitWaitTimeout)` - 等待超时。
///
/// ## 性能提示
/// - `retry_interval` 不宜过短，以免频繁调用系统API造成性能损耗。
/// - `wait_timeout` 应根据实际需求合理设置，避免无限等待。
async fn wait_for_process_exit(
    pid: i32,
    wait_timeout: Duration,
    retry_interval: Duration,
) -> Result<(), ProcessError> {
    timeout(wait_timeout, async move {
        Ok(while check_process(pid)? {
            tokio::time::sleep(retry_interval).await;
        })
    })
    .await
    .map_err(|_| ProcessError::TerminateProcessTimeout(pid))?
}

/// # 检查进程是否存在
///
/// 通过发送信号0来检查指定PID的进程是否存在。信号0不会真正发送信号，仅用于验证进程状态。
///
/// ## 参数
///
/// * `pid` - 要检查的进程ID。
///
/// ## 返回值
///
/// * `Ok(true)` - 进程存在。
/// * `Ok(false)` - 进程不存在。
/// * `Err(CheckProcessError)` - 检查过程中发生错误。
///
/// ## 安全性说明
///
/// 此函数使用 `unsafe` 块调用系统级API（`libc::kill`），但已被妥善封装以确保内存安全。
/// 调用者无需担心未定义行为或内存泄漏问题。
///
/// ## 错误类型
/// - `ESRCH`: 进程不存在。
/// - `EPERM`: 进程存在但无权限访问。
/// - 其他错误: 返回具体错误信息。
pub fn check_process(pid: i32) -> Result<bool, ProcessError> {
    unsafe {
        let result = libc::kill(pid, 0); // 信号 0
        if result == 0 {
            Ok(true) // 进程存在
        } else {
            let err = io::Error::last_os_error();
            match err.raw_os_error() {
                Some(libc::ESRCH) => Ok(false), // 进程不存在
                Some(libc::EPERM) => Ok(true),  // 进程存在但无权限
                Some(e) => Err(ProcessError::CheckProcess(e.to_string())),
                None => Err(ProcessError::CheckProcess("Unknown error".to_string())),
            }
        }
    }
}
