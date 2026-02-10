//! # PID 管理模块
//!
//! 提供进程ID相关的管理功能，包括PID文件的读取、写入和删除操作。
//! 主要用于确保进程的唯一性和状态跟踪。

pub(super) mod pid_error;
pub(super) mod pid_file_guard;
pub(super) mod pid_utils;
