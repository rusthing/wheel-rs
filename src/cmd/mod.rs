//! # 命令行工具模块
//!
//! 提供执行外部命令的实用工具函数，分为同步与异步两套实现：
//! - [`std`]：基于 `std::process` 的同步实现
//! - [`spawn`]：基于 `tokio::process` 的异步实现
//!
//! 进程管理（存活检查、终止、信号）相关的功能在 [`crate::process`] 模块。

pub mod cmd_error;
pub mod spawn;
pub mod std;
