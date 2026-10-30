//! # wheel-rs
//!
//! 一个 Rust 工具库：文件/时间/DNS/网络地址处理、命令行与进程管理、压缩前缀树，
//! 以及一组自定义 serde 实现。
//!
//! ## 功能特性
//!
//! - **文件工具**: 文件扩展名提取、SHA256 哈希计算、跨设备错误识别、带去抖动的文件监听
//! - **时间工具**: 当前时间戳、时长字符串互转、定时器构建，以及外部时间戳
//!   的规范化校验（[`time_utils::normalize_ms`]）
//! - **DNS / 网络工具**: 主机名与 `host[:port]` 解析、本机 IP 获取、网段比较
//! - **命令与进程**: 同步/异步执行外部命令、PID 文件管理、进程存活检查与信号发送
//! - **数据结构**: 压缩前缀树（[`trie`]）、URN 匹配
//! - **序列化工具**: 为 `std::time::Duration`、`u64`、`IpNet`、`PathBuf` 等类型
//!   提供自定义序列化和反序列化支持
//!
//! ## 模块说明
//!
//! - [`addr_utils`] — 网络地址（主机 + 端口）解析与表示工具
//! - [`file_utils`] — 文件操作工具函数
//! - [`time_utils`] — 时间相关工具函数（含外部时间戳的规范化校验）
//! - [`dns_utils`] — DNS 解析工具函数
//! - [`config_utils`] — 配置差异比较工具
//! - [`ipnet_utils`] — IP 网络比较与精确性判断工具
//! - [`str_utils`] — 字符串格式转换工具
//! - [`trie`] — 压缩前缀树（Patricia Trie）
//! - [`urn_utils`] — URN 解析与表示工具
//! - [`cmd`] — 命令行执行工具
//! - [`process`] — 进程管理工具
//! - [`serde`] — 自定义序列化/反序列化实现

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
pub mod trie;
pub mod urn_utils;
