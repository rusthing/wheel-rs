//! # 自定义序列化/反序列化器，用于处理 `Option<ResolveHosts>` 类型的数据
//!
//! 此模块提供 `Option<ResolveHosts>` 类型的自定义序列化和反序列化实现。
//! 序列化时将 `ResolveHosts` 枚举映射为字符串（"Auto"/"Always"/"Never"），`None` 映射为 null；
//! 反序列化时不区分大小写。

use hickory_resolver::config::ResolveHosts;
use serde::de::Unexpected;
use serde::{Deserialize, Deserializer, Serializer};

/// # 将 `Option<ResolveHosts>` 序列化为字符串或 null
///
/// - `Some(ResolveHosts::Auto)` → `"Auto"`
/// - `Some(ResolveHosts::Always)` → `"Always"`
/// - `Some(ResolveHosts::Never)` → `"Never"`
/// - `None` → `null`
///
/// ## 参数
/// - `resolve_hosts`: 待序列化的值。
/// - `serializer`: 序列化器。
///
/// ## 返回值
/// 返回序列化结果，序列化失败时返回 `S::Error`。
pub fn serialize<S>(resolve_hosts: &Option<ResolveHosts>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match resolve_hosts {
        Some(rh) => {
            let s = match rh {
                ResolveHosts::Auto => "Auto",
                ResolveHosts::Always => "Always",
                ResolveHosts::Never => "Never",
            };
            serializer.serialize_str(s)
        }
        None => serializer.serialize_none(),
    }
}

/// # 从字符串或 null 反序列化为 `Option<ResolveHosts>`
///
/// 支持不区分大小写的字符串："auto"、"always"、"never"；`null` 返回 `None`。
///
/// ## 参数
/// - `deserializer`: 反序列化器。
///
/// ## 返回值
/// 返回反序列化后的值；字符串无法识别时返回 `D::Error`。
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ResolveHosts>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => Ok(Some(match s.to_lowercase().as_str() {
            "auto" => ResolveHosts::Auto,
            "always" => ResolveHosts::Always,
            "never" => ResolveHosts::Never,
            _ => Err(serde::de::Error::invalid_value(
                Unexpected::Str(&s),
                &"auto, always, never",
            ))?,
        })),
        None => Ok(None),
    }
}
