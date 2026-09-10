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
//! - [addr_utils]: 网络地址（主机 + 端口）解析与表示工具
//! - [file_utils]: 文件操作工具函数
//! - [time_utils]: 时间相关工具函数
//! - [dns_utils]: DNS 解析工具函数
//! - [config_utils]: 配置差异比较工具
//! - [ipnet_utils]: IP 网络比较与精确性判断工具
//! - [str_utils]: 字符串格式转换工具
//! - [urn_utils]: URN 解析与表示工具
//! - [cmd]: 命令行执行工具
//! - [process]: 进程管理工具
//! - [serde]: 自定义序列化/反序列化实现

pub mod addr_utils;
pub mod cmd;
pub mod config_utils;
pub mod dns_utils;
pub mod file_utils;
pub mod ipnet_utils;
pub mod process;
pub mod serde;
pub mod str_utils;
pub mod time_utils;
pub mod urn_utils;
