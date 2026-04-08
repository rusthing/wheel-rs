//! # 序列化模块
//!
//! 提供各种自定义序列化和反序列化实现。
//!
//! 该模块包含以下子模块：
//! - [duration_option_serde] - 为 `Option<Duration>` 提供自定义序列化
//! - [duration_serde] - 为 `Duration` 提供自定义序列化
//! - [log_filter_serde] - 为 `Option<LevelFilter>` 提供自定义序列化
//! - [u64_option_serde] - 为 `Option<u64>` 提供自定义序列化，支持字符串和数字格式
//! - [u64_serde] - 为 `u64` 提供自定义序列化，支持字符串和数字格式
//! - [vec_option_serde] - 为 `Option<Vec<String>>` 提供自定义序列化

pub mod duration_option_serde;
pub mod duration_serde;
pub mod log_filter_serde;
pub mod option_option_serde;
pub mod path_buf_option_serde;
pub mod path_buf_serde;
pub mod rotation_serde;
pub mod u64_option_serde;
pub mod u64_serde;
pub mod vec_option_serde;
pub mod vec_serde;
pub mod vec_urn_serde;
