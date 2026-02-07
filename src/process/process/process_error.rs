//! # 进程错误类型定义
//!
//! 定义进程管理过程中可能出现的各种错误类型，用于统一处理进程检查、退出等操作中的异常情况。
//! 该模块通过 `thiserror` 提供结构化的错误类型，方便上层业务逻辑进行模式匹配和错误传播。

use crate::process::SignalError;
use thiserror::Error;

/// # 进程相关错误枚举
///
/// 包含进程管理过程中可能发生的各种错误类型，适用于进程检查、退出等待等场景。
/// 通过 `thiserror` 宏实现，支持自动派生 `Display` 和 `Debug` 特性。
#[derive(Error, Debug)]
pub enum ProcessError {
    /// 检查进程失败错误
    ///
    /// 当尝试检查进程状态（如是否存在、是否运行）时发生错误。
    /// 可能的原因包括权限不足、进程 ID 无效或系统调用失败。
    ///
    /// ## 示例
    /// ```rust
    /// use crate::process::ProcessError;
    /// let error = ProcessError::CheckProcessError("Permission denied".to_string());
    /// ```
    #[error("Fail to check process: {0}")]
    CheckProcess(String),

    #[error("{0}")]
    Signal(#[from] SignalError),

    /// 进程退出等待超时
    ///
    /// 当等待进程退出时超过预设时间限制，触发此错误。
    /// 通常发生在进程未能在预期时间内终止的情况下。
    ///
    /// ## 参数
    /// - `pid`: 超时未退出的进程 ID。
    ///
    /// ## 示例
    /// ```rust
    /// use crate::process::ProcessError;
    /// let error = ProcessError::ProcessExitWaitTimeout(1234);
    /// ```
    #[error("Process exit wait timeout: pid-{0}")]
    TerminateProcessTimeout(i32),
}
