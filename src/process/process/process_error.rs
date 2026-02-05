//! # 进程错误类型定义
//!
//! 定义进程管理过程中可能出现的各种错误类型。

use thiserror::Error;

/// # 进程相关错误枚举
///
/// 包含进程检查和退出过程中可能发生的各种错误。
#[derive(Error, Debug)]
pub enum ProcessError {
    /// 检查进程失败错误
    #[error("Fail to check process: {0}")]
    CheckProcessError(String),
    /// 进程退出失败错误
    #[error("Process exit failed: pid-{0}")]
    ProcessExitError(i32),
}