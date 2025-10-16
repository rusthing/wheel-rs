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
    match dur {
        Some(d) => serializer.serialize_str(&format!("{}s", d.as_secs())),
        None => serializer.serialize_none(),
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
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => Ok(Some(
            parse_duration(&s).expect("不正确的Duration字符串，支持的格式如5s、3m、6h"),
        )),
        None => Ok(None),
    }
}
