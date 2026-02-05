//! # 进程管理工具函数
//!
//! 提供进程终止、状态检查等核心功能的实用工具函数。

use crate::process::ProcessError::{CheckProcessError, ProcessExitError};
use crate::process::{send_signal_by_instruction, ProcessError};
use std::io;
use std::time::Duration;

/// # 终止进程
///
/// 发送终止信号给指定进程并等待其退出。
///
/// ## 参数
///
/// * `pid` - 目标进程ID
/// * `max_retries` - 最大重试次数
/// * `retry_interval` - 重试间隔时间
///
/// ## 返回值
///
/// 成功时返回 `Ok(())`，失败时返回相应的错误。
///
/// ## 错误处理
///
/// 如果进程在最大重试次数后仍未退出，将返回 [ProcessExitError] 错误。
pub async fn terminate_process(
    pid: i32,
    max_retries: u32,
    retry_interval: Duration,
) -> Result<(), ProcessError> {
    send_signal_by_instruction("terminate", pid).expect("Failed to send signal: ");
    wait_for_process_exit(pid, max_retries, retry_interval).await?;
    Ok(())
}

/// # 等待进程退出
///
/// 循环检查进程是否存在，直到进程退出或达到最大重试次数。
///
/// ## 参数
///
/// * `pid` - 目标进程ID
/// * `max_retries` - 最大重试次数
/// * `retry_interval` - 重试间隔时间
///
/// ## 返回值
///
/// 成功时返回 `Ok(())`，如果超过重试次数则返回 [ProcessExitError] 错误。
async fn wait_for_process_exit(
    pid: i32,
    max_retries: u32,
    retry_interval: Duration,
) -> Result<(), ProcessError> {
    let mut current_retry_count = 0;
    while check_process(pid)? {
        // 进程仍然存在，继续等待
        current_retry_count += 1;
        if current_retry_count >= max_retries {
            Err(ProcessExitError(pid))?
        }
        tokio::time::sleep(retry_interval).await;
    }
    Ok(())
}

/// # 检查进程是否存在
///
/// 通过发送信号0来检查指定PID的进程是否存在。
///
/// ## 参数
///
/// * `pid` - 要检查的进程ID
///
/// ## 返回值
///
/// 返回 `Ok(true)` 表示进程存在，`Ok(false)` 表示进程不存在，
/// 错误时返回 [CheckProcessError]。
///
/// ## 安全性说明
///
/// 此函数使用 `unsafe` 块调用系统级API，但已被妥善封装以确保内存安全。
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
                Some(e) => Err(CheckProcessError(e.to_string())),
                None => Err(CheckProcessError("Unknown error".to_string())),
            }
        }
    }
}