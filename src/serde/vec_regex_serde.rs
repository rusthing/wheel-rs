//! # 自定义序列化/反序列化器，用于处理 `Vec<Regex>` 类型的数据
//!
//! 此模块提供 `Vec<Regex>` 类型的自定义序列化和反序列化实现，
//! 支持将单个正则表达式字符串或字符串数组转换为正则向量。

use regex::Regex;
use serde::{
    de::{self, Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
};
use std::fmt;

/// # 将 `Vec<Regex>` 序列化为字符串数组
///
/// 将正则表达式向量中的每个元素以原始字符串形式序列化为 JSON 数组。
///
/// ## 参数
/// - `vec`: 待序列化的正则向量。
/// - `serializer`: 序列化器。
///
/// ## 返回值
/// 返回序列化结果，序列化失败时返回 `S::Error`。
pub fn serialize<S>(vec: &Vec<Regex>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(vec.len()))?;
    for element in vec {
        seq.serialize_element(element.as_str())?;
    }
    seq.end()
}

/// # 将字符串或字符串数组反序列化为 `Vec<Regex>`
///
/// 支持的输入格式：
/// - 单个字符串 `"^foo"` → 单元素正则向量
/// - 字符串数组 `["^foo", "bar$"]` → 对应正则向量
///
/// ## 参数
/// - `deserializer`: 反序列化器。
///
/// ## 返回值
/// 返回反序列化后的正则向量；存在无效的正则表达式时返回 `D::Error`。
pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Regex>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(RegexVecVisitor)
}

struct RegexVecVisitor;

impl<'de> Visitor<'de> for RegexVecVisitor {
    type Value = Vec<Regex>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a regex string or array of regex strings")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let regex =
            Regex::new(value).map_err(|e| de::Error::custom(format!("invalid regex: {e}")))?;
        Ok(vec![regex])
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
            let regex = Regex::new(&element)
                .map_err(|e| de::Error::custom(format!("invalid regex '{element}': {e}")))?;
            vec.push(regex);
        }
        Ok(vec)
    }
}
