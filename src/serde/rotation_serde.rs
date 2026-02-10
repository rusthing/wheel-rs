//! # 日志轮转序列化模块
//!
//! 提供对 `tracing_appender::rolling::Rotation` 枚举类型的自定义序列化和反序列化实现。
//! 支持将轮转策略序列化为字符串格式（如 "daily"、"hourly" 等）。
//!
//! ## 示例
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use tracing_appender::rolling::Rotation;
//! 
//! #[derive(Serialize, Deserialize)]
//! struct Config {
//!     #[serde(with = "wheel_rs::serde::rotation_serde")]
//!     log_rotation: Rotation,
//! }
//! ```

use serde::de::Unexpected;
use serde::{Deserialize, Deserializer, Serializer};
use tracing_appender::rolling::Rotation;

pub fn serialize<S>(value: &Rotation, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let rotation_str = match value {
        &Rotation::WEEKLY => "weekly",
        &Rotation::DAILY => "daily",
        &Rotation::HOURLY => "hourly",
        &Rotation::MINUTELY => "minutely",
        &Rotation::NEVER => "never",
    };
    serializer.serialize_str(rotation_str)
}

/// # 反序列化轮转策略
///
/// 将字符串反序列化为 `Rotation` 枚举值。
///
/// ## 支持的字符串映射
///
/// * "weekly" → `Rotation::WEEKLY`
/// * "daily" → `Rotation::DAILY`
/// * "hourly" → `Rotation::HOURLY`
/// * "minutely" → `Rotation::MINUTELY`
/// * "never" → `Rotation::NEVER`
///
/// ## 参数
///
/// * `deserializer` - 反序列化器
///
/// ## 返回值
///
/// 返回反序列化后的轮转策略或错误
///
/// ## 错误处理
///
/// 如果输入的字符串不是有效的轮转策略，将返回 `invalid_value` 错误，
/// 并提示有效的选项列表。
///
/// ## 示例
///
/// ```rust
/// use serde_json;
/// use tracing_appender::rolling::Rotation;
/// use wheel_rs::serde::rotation_serde;
///
/// let json = "\"daily\"";
/// let deserialized: Rotation = serde_json::from_str(json).unwrap();
/// assert_eq!(deserialized, Rotation::DAILY);
/// ```
pub fn deserialize<'de, D>(deserializer: D) -> Result<Rotation, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    let log_rotation = s.trim().to_lowercase();
    Ok(match log_rotation.as_str() {
        "weekly" => Rotation::WEEKLY,
        "daily" => Rotation::DAILY,
        "hourly" => Rotation::HOURLY,
        "minutely" => Rotation::MINUTELY,
        "never" => Rotation::NEVER,
        _ => {
            return Err(serde::de::Error::invalid_value(
                Unexpected::Str(&log_rotation),
                &"weekly/daily/hourly/minutely/never",
            ));
        }
    })
}
