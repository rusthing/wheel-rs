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

/// # 序列化轮转策略
///
/// 将 `Rotation` 枚举值序列化为其小写字符串形式：
///
/// * `Rotation::WEEKLY` → `"weekly"`
/// * `Rotation::DAILY` → `"daily"`
/// * `Rotation::HOURLY` → `"hourly"`
/// * `Rotation::MINUTELY` → `"minutely"`
/// * `Rotation::NEVER` → `"never"`
///
/// ## 参数
///
/// * `value` - 待序列化的轮转策略
/// * `serializer` - 序列化器
///
/// ## 返回值
///
/// 返回序列化结果，失败时透传序列化器错误。
///
/// ## 示例
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use tracing_appender::rolling::Rotation;
///
/// #[derive(Serialize, Deserialize)]
/// struct Config {
///     #[serde(with = "wheel_rs::serde::rotation_serde")]
///     log_rotation: Rotation,
/// }
///
/// let cfg = Config { log_rotation: Rotation::DAILY };
/// assert_eq!(serde_json::to_string(&cfg).unwrap(), r#"{"log_rotation":"daily"}"#);
/// ```
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
/// 输入会先经 `trim()` 与 `to_lowercase()` 处理，因此**大小写不敏感且容忍首尾空白**，
/// 例如 `"DAILY"`、`" daily "` 都会被解析为 `Rotation::DAILY`。
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
/// 需通过 `#[serde(with = "...")]` 标注字段才会生效。
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use tracing_appender::rolling::Rotation;
///
/// #[derive(Serialize, Deserialize)]
/// struct Config {
///     #[serde(with = "wheel_rs::serde::rotation_serde")]
///     log_rotation: Rotation,
/// }
///
/// let cfg: Config = serde_json::from_str(r#"{"log_rotation":"daily"}"#).unwrap();
/// assert_eq!(cfg.log_rotation, Rotation::DAILY);
///
/// // 大小写不敏感
/// let cfg: Config = serde_json::from_str(r#"{"log_rotation":"HOURLY"}"#).unwrap();
/// assert_eq!(cfg.log_rotation, Rotation::HOURLY);
///
/// // 非法值返回 Err
/// assert!(serde_json::from_str::<Config>(r#"{"log_rotation":"yearly"}"#).is_err());
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
