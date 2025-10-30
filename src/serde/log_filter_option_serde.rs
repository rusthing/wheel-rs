//! # 日志级别过滤器的序列化与反序列化
//!
//! 该模块为 [`LevelFilter`] 类型提供自定义的序列化和反序列化函数，
//! 使其能够与 serde 兼容的数据格式（如 JSON、TOML、YAML 等）一起使用。
//!
//! 实现将 LevelFilter 序列化为小写字符串：
//! - `Off` => "off"
//! - `Error` => "error"
//! - `Warn` => "warn"
//! - `Info` => "info"
//! - `Debug` => "debug"
//! - `Trace` => "trace"
//!
//! 反序列化时不区分大小写，可接受上述字符串的任意大小写形式。
//!
//! ## 示例
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use log::LevelFilter;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Config {
//!     #[serde(with = "wheel_rs::serde::log_filter_serde")]
//!     log_level: LevelFilter,
//! }
//! ```

use log::LevelFilter;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// # 将 `LevelFilter` 序列化为字符串表示形式。
///
/// 此函数接收一个 `LevelFilter` 和一个序列化器，将日志级别过滤器序列化为其对应
/// 的字符串表示形式（例如："info"、"debug"、"error" 等）。
///
/// ## 参数
///
/// * `level` - 需要序列化的 `LevelFilter` 的引用
/// * `serializer` - 用于序列化的序列化器
///
/// ## 返回值
///
/// 返回包含序列化结果的 `Result`，如果序列化失败则返回错误。
pub fn serialize<S>(level: &Option<LevelFilter>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(level) = level {
        level.as_str().serialize(serializer)
    } else {
        serializer.serialize_none()
    }
}

/// # 将字符串反序列化为 `LevelFilter`。
///
/// 此函数接收一个反序列化器，将字符串形式的日志级别反序列化为对应的
/// `LevelFilter` 枚举值。
///
/// ## 支持的级别字符串
///
/// - "off"   - 关闭所有日志
/// - "error" - 错误级别
/// - "warn"  - 警告级别
/// - "info"  - 信息级别
/// - "debug" - 调试级别
/// - "trace" - 跟踪级别
///
/// 字符串比较不区分大小写。
///
/// ## 参数
///
/// * `deserializer` - 用于反序列化的反序列化器
///
/// ## 返回值
///
/// 返回包含反序列化结果的 `Result`，如果字符串无法匹配任何已知的日志级别，
/// 则返回自定义错误。
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<LevelFilter>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = s {
        Ok(Some(match s.to_lowercase().as_str() {
            "off" => LevelFilter::Off,
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "Unknown log level: {}",
                    s
                )));
            }
        }))
    } else {
        Ok(None)
    }
}
