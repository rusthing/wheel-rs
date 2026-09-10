//! # 自定义序列化/反序列化器，用于处理 `Vec<Urn>` 类型的数据
//!
//! 此模块提供 `Vec<Urn>` 类型的自定义序列化和反序列化实现，
//! 支持将单个字符串、逗号分隔字符串或字符串数组转换为 URN 向量。

use crate::urn_utils::Urn;
use serde::{
    de::{self, Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
};
use std::fmt;

/// # 将 `Vec<Urn>` 序列化为字符串数组
///
/// 将 URN 向量中的每个元素以字符串形式序列化为 JSON 数组。
///
/// ## 参数
/// - `vec`: 待序列化的 URN 向量。
/// - `serializer`: 序列化器。
///
/// ## 返回值
/// 返回序列化结果，序列化失败时返回 `S::Error`。
pub fn serialize<S>(vec: &Vec<Urn>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(vec.len()))?;
    for element in vec {
        seq.serialize_element(&element.to_string())?;
    }
    seq.end()
}

/// # 将字符串或字符串数组反序列化为 `Vec<Urn>`
///
/// 支持的输入格式：
/// - 单个字符串 `"urn:example:foo"` → 单元素 URN 向量
/// - 逗号分隔字符串 `"urn:example:foo,urn:example:bar"` → 对应 URN 向量
/// - 字符串数组 `["urn:example:foo", "urn:example:bar"]` → 对应 URN 向量
///
/// ## 参数
/// - `deserializer`: 反序列化器。
///
/// ## 返回值
/// 返回反序列化后的 URN 向量；存在无法解析的 URN 元素时返回 `D::Error`。
pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Urn>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(UrnVecVisitor)
}

struct UrnVecVisitor;

impl<'de> Visitor<'de> for UrnVecVisitor {
    type Value = Vec<Urn>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string, comma-separated string, or array of strings")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(value.to_string())
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| Urn::from_str(&s).map_err(|e| de::Error::custom(format!("{e:?}"))))
            .collect::<Result<Vec<_>, _>>()?)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        while let Some(element) = seq.next_element::<String>()? {
            let element = element.trim().to_string();
            if element.is_empty() {
                continue;
            }
            vec.push(Urn::from_str(&element).map_err(|e| de::Error::custom(format!("{e:?}")))?);
        }
        Ok(vec)
    }
}
