//! # 序列化和反序列化 Duration 类型
//!
//! 此模块提供了对 Duration 类型的自定义序列化和反序列化实现。
//! 序列化时将 Duration 转换为字符串格式（如 "5s" 表示5秒），
//! 反序列化时将字符串解析为 Duration。
//!
//! ## 示例
//!
//! ```
//! use std::time::Duration;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Config {
//!     #[serde(with = "wheel_rs::serde::duration_serde")]
//!     timeout: Duration,
//! }
//! ```

use humantime::parse_duration;
use serde::{Deserialize, Deserializer, Serializer};
use std::time::Duration;

/// # Duration序列化
///
/// 将 Duration 序列化为字符串格式。Duration 会被转换为以秒为单位的字符串，
/// 格式为数字后跟 's' 字符（例如 "5s" 表示5秒）。
///
/// ## 示例
///
/// 必须通过 `#[serde(with = "...")]` 标注字段才会生效；直接序列化裸 `Duration`
/// 走的是 serde 默认实现（输出 `{"secs":..,"nanos":..}`），不会经过本模块。
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use std::time::Duration;
///
/// #[derive(Serialize, Deserialize)]
/// struct Config {
///     #[serde(with = "wheel_rs::serde::duration_serde")]
///     timeout: Duration,
/// }
///
/// let cfg = Config { timeout: Duration::from_secs(5) };
/// assert_eq!(serde_json::to_string(&cfg).unwrap(), r#"{"timeout":"5s"}"#);
/// ```
pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{}s", duration.as_secs()))
}

/// # Duration反序列化
///
/// 将字符串格式的时间间隔反序列化为 Duration 类型。
/// 支持的格式包括 "5s"（5秒）、"3m"（3分钟）、"6h"（6小时）等。
///
/// ## 示例
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use std::time::Duration;
///
/// #[derive(Serialize, Deserialize)]
/// struct Config {
///     #[serde(with = "wheel_rs::serde::duration_serde")]
///     timeout: Duration,
/// }
///
/// let cfg: Config = serde_json::from_str(r#"{"timeout":"5s"}"#).unwrap();
/// assert_eq!(cfg.timeout, Duration::from_secs(5));
///
/// // humantime 支持的写法均可解析
/// let cfg: Config = serde_json::from_str(r#"{"timeout":"3m"}"#).unwrap();
/// assert_eq!(cfg.timeout, Duration::from_secs(180));
/// ```
pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    parse_duration(&s).map_err(serde::de::Error::custom)
}
