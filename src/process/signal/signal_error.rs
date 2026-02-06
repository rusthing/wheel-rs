//! # 信号错误类型定义
//!
//! 定义信号发送过程中可能出现的各种错误类型，用于统一处理信号指令无效或发送失败等异常情况。
//! 该模块通过 `thiserror` 提供结构化的错误类型，方便上层业务逻辑进行模式匹配和错误传播。
use thiserror::Error;

/// # 信号相关错误枚举
///
/// 包含信号处理过程中可能发生的各种错误类型，适用于信号指令验证、信号发送等场景。
/// 通过 `thiserror` 宏实现，支持自动派生 `Display` 和 `Debug` 特性。
#[derive(Error, Debug)]
pub enum SignalError {
    /// 无效指令错误
    ///
    /// 当传入的信号指令不符合预期格式或未被支持时触发此错误。
    ///
    /// ## 参数
    /// - `instruction`: 无效的指令字符串。
    ///
    /// ## 示例
    /// ```rust
    /// use crate::signal::SignalError;
    /// let error = SignalError::InvalidInstructionError("unknown_signal".to_string());
    /// ```
    #[error("Invalid instruction: {0}")]
    InvalidInstructionError(String),

    /// 发送信号失败错误
    ///
    /// 当尝试发送信号时因权限不足、目标进程不存在或其他系统级原因导致失败时触发此错误。
    ///
    /// ## 参数
    /// - `reason`: 失败的具体原因描述。
    ///
    /// ## 示例
    /// ```rust
    /// use crate::signal::SignalError;
    /// let error = SignalError::SendSignalError("Permission denied".to_string());
    /// ```
    #[error("Fail to send signal: {0}")]
    SendSignalError(String),
}
