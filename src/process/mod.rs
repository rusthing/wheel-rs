//! # 进程管理模块
//!
//! 提供进程管理相关的核心功能，包括：
//! - PID 文件的读取、写入、删除与生命周期管理
//! - 进程的终止与存在性检查
//! - 系统信号的发送与异步监听
//!
//! 通过 `pub use` 将各子模块中的公开项统一重导出到本模块，
//! 外部可通过 `wheel_rs::process::*` 直接引用。

mod pid;
mod process;
mod signal;

// 重新导出结构体，简化外部引用
pub use pid::pid_error::*;
pub use pid::pid_file_guard::*;
pub use pid::pid_utils::*;
pub use process::process_error::*;
pub use process::process_utils::*;
pub use signal::signal_error::*;
pub use signal::signal_utils::*;
