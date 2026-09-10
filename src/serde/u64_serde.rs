//! # 自定义序列化/反序列化器，用于处理 `u64` 类型的数据
//!
//! 此模块提供 `u64` 类型的自定义序列化和反序列化实现。
//! 序列化时数字转为字符串，反序列化时兼容字符串和数字两种输入格式。

use serde::{Deserializer, Serialize, Serializer};

/// # u64 序列化为字符串
///
/// 将u64 类型序列化为字符串格式，便于 JSON 传输和存储
pub fn serialize<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    value.to_string().serialize(serializer)
}

/// # 从字符串或数字反序列化为 u64
///
/// 支持从字符串或数字两种格式反序列化为 u64 类型
/// - 如果源数据是字符串，尝试解析为 u64
/// - 如果源数据是数字，直接转换为 u64
pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    // 定义访问器来处理不同类型的输入
    struct U64Visitor;

    impl<'de> serde::de::Visitor<'de> for U64Visitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or unsigned integer")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if value < 0 {
                Err(E::custom(format!(
                    "negative value {} cannot be converted to u64",
                    value
                )))
            } else {
                Ok(value as u64)
            }
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.parse::<u64>().map_err(E::custom)
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.parse::<u64>().map_err(E::custom)
        }
    }

    deserializer.deserialize_any(U64Visitor)
}
