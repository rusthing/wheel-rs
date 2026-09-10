//! # 序列化和反序列化 `Option<Option<Duration>>` 类型
//!
//! 此模块提供了对 `Option<Option<Duration>>` 类型的自定义序列化和反序列化实现，
//! 用于区分"字段不存在"（外层 `None`）与"字段存在但值为空"（`Some(None)`）两种情况。
//! 序列化时将内层的 `Duration` 转换为以秒为单位的字符串（如 "5s" 表示 5 秒），
//! 反序列化时将字符串解析为 `Duration`。

use serde::{Deserialize, Deserializer, Serializer};
use std::time::Duration;

/// # 序列化 `Option<Option<Duration>>`
///
/// 将 `Option<Option<Duration>>` 序列化为字符串或 null：
/// - `Some(Some(d))` → 字符串 `"{secs}s"`，其中 `secs` 为以秒为单位的值
/// - `Some(None)` 或 `None` → `null`
///
/// ## 参数
/// - `value`: 待序列化的值。
/// - `serializer`: 序列化器。
///
/// ## 返回值
/// 返回序列化结果，序列化失败时返回 `S::Error`。
pub fn serialize<S>(value: &Option<Option<Duration>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(Some(d)) => serializer.serialize_str(&format!("{}s", d.as_secs())),
        Some(None) | None => serializer.serialize_none(),
    }
}

/// # 反序列化 `Option<Option<Duration>>`
///
/// 从字符串或 null 反序列化为 `Option<Option<Duration>>`：
/// - `null` → `None`（字段不存在）
/// - 内层为 null → `Some(None)`（字段存在但值为空）
/// - 字符串（如 `"5s"`）→ `Some(Some(Duration))`
///
/// 支持的字符串格式与 `humantime` 一致，如 "5s"、"3m"、"6h" 等。
///
/// ## 参数
/// - `d`: 反序列化器。
///
/// ## 返回值
/// 返回反序列化后的值，字符串无法解析为时长时返回 `D::Error`。
pub fn deserialize<'de, D>(d: D) -> Result<Option<Option<Duration>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<Option<String>> = Option::<Option<String>>::deserialize(d)?;
    match opt {
        None => Ok(None),
        Some(None) => Ok(Some(None)),
        Some(Some(s)) => {
            let dur = humantime::parse_duration(&s).map_err(serde::de::Error::custom)?;
            Ok(Some(Some(dur)))
        }
    }
}
