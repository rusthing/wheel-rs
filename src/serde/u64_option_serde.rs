use serde::{Deserializer, Serialize, Serializer};

/// # Option<u64> 序列化为字符串或 null
///
/// 将 Option<u64> 类型序列化为字符串格式或 null，便于 JSON 传输和存储
/// - Some(value) 序列化为字符串 "value"
/// - None 序列化为 null
pub fn serialize<S>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(v) => v.to_string().serialize(serializer),
        None => serializer.serialize_none(),
    }
}

/// # 从字符串、数字或 null 反序列化为 Option<u64>
///
/// 支持从多种格式反序列化为 Option<u64> 类型
/// - 如果源数据是字符串，尝试解析为 u64，返回 Some(value)
/// - 如果源数据是数字，直接转换为 u64，返回 Some(value)
/// - 如果源数据是 null，返回 None
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    // 定义访问器来处理不同类型的输入
    struct OptionU64Visitor;

    impl<'de> serde::de::Visitor<'de> for OptionU64Visitor {
        type Value = Option<u64>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("null, a string, or unsigned integer")
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
            // 委托给内部的 u64 反序列化
            struct InnerVisitor;

            impl<'de> serde::de::Visitor<'de> for InnerVisitor {
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

            deserializer.deserialize_any(InnerVisitor).map(Some)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_option(OptionU64Visitor)
}
