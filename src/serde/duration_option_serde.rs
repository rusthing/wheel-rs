//! # 序列化和反序列化 Duration 类型
//!
//! 此模块提供了对 Option<Duration> 类型的自定义序列化和反序列化实现。
//! 序列化时将 Duration 转换为字符串格式（如 "5s" 表示5秒），
//! 反序列化时将字符串解析为 Duration。
use humantime::parse_duration;
use serde::{Deserialize, Deserializer, Serializer};
use std::time::Duration;

/// # Duration序列化
///
/// 将 Option<Duration> 序列化为字符串格式。Some(Duration) 会被转换为以秒为单位的字符串，
/// 格式为数字后跟 's' 字符（例如 "5s" 表示5秒）。None 值会被序列化为 null。
pub fn serialize<S>(dur: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(d) = dur {
        serializer.serialize_str(&format!("{}s", d.as_secs()))
    } else {
        serializer.serialize_none()
    }
}

/// # Duration反序列化
///
/// 将字符串格式的时间间隔反序列化为 Option<Duration> 类型。
/// 支持的格式包括 "5s"（5秒）、"3m"（3分钟）、"6h"（6小时）等。
/// 如果输入为 None，则返回 None。
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_option(DurationVisitor)
}

struct DurationVisitor;
impl<'de> serde::de::Visitor<'de> for DurationVisitor {
    type Value = Option<Duration>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Invalid duration string, supported formats like 5s, 3m, 6h")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        parse_duration(&s)
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
}
