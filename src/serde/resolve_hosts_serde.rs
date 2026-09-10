//! # 自定义序列化/反序列化器，用于处理 `ResolveHosts` 类型的数据
//!
//! 此模块提供 `ResolveHosts` 类型的自定义序列化和反序列化实现。
//! 序列化时将 `ResolveHosts` 枚举映射为字符串（"Auto"/"Always"/"Never"）；
//! 反序列化时不区分大小写。

use hickory_resolver::config::ResolveHosts;
use serde::de::Unexpected;
use serde::{Deserialize, Deserializer};

/// # 将 `ResolveHosts` 序列化为字符串
///
/// - `ResolveHosts::Auto` → `"Auto"`
/// - `ResolveHosts::Always` → `"Always"`
/// - `ResolveHosts::Never` → `"Never"`
///
/// ## 参数
/// - `resolve_hosts`: 待序列化的值。
/// - `serializer`: 序列化器。
///
/// ## 返回值
/// 返回序列化结果，序列化失败时返回 `S::Error`。
pub fn serialize<S>(resolve_hosts: &ResolveHosts, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = match resolve_hosts {
        ResolveHosts::Auto => "Auto",
        ResolveHosts::Always => "Always",
        ResolveHosts::Never => "Never",
    };
    serializer.serialize_str(s)
}

/// # 从字符串反序列化为 `ResolveHosts`
///
/// 支持不区分大小写的字符串："auto"、"always"、"never"。
///
/// ## 参数
/// - `deserializer`: 反序列化器。
///
/// ## 返回值
/// 返回反序列化后的值；字符串无法识别时返回 `D::Error`。
pub fn deserialize<'de, D>(deserializer: D) -> Result<ResolveHosts, D::Error>
where
    D: Deserializer<'de>,
{
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.to_lowercase().as_str() {
            "auto" => ResolveHosts::Auto,
            "always" => ResolveHosts::Always,
            "never" => ResolveHosts::Never,
            _ => Err(serde::de::Error::invalid_value(
                Unexpected::Str(&s),
                &"auto, always, never",
            ))?,
        })
    }
}
