//! # 同步命令执行子模块
//!
//! 基于 `std::process` 提供同步执行外部命令、检查进程存活与杀死进程的工具。

pub mod cmd_utils;

// 重新导出结构体，简化外部引用
pub use cmd_utils::*;
