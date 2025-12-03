//! # wheel-rs
//!
//! 一个 Rust 工具库，提供文件操作、时间工具和 Duration 序列化支持。
//!
//! ## 功能特性
//!
//! - **文件工具**: 提供文件扩展名提取和 SHA256 哈希值计算功能
//! - **时间工具**: 提供时间戳和时间测量相关工具
//! - **DNS 工具**: 提供 DNS 解析功能
//! - **命令行工具**: 提供执行外部命令的功能
//! - **序列化工具**: 为 `std::time::Duration` 和其他类型提供自定义序列化和反序列化支持
//!
//! ## 模块说明
//!
//! - [file_utils]: 文件操作工具函数
//! - [time_utils]: 时间相关工具函数
//! - [dns_utils]: DNS 解析工具函数
//! - [cmd]: 命令行执行工具
//! - [serde]: 自定义序列化/反序列化实现

pub mod cmd;
pub mod dns_utils;
pub mod file_utils;
pub mod serde;
pub mod time_utils;
pub mod urn_utils;
