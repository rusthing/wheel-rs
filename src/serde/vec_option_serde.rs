//! # 自定义反序列化器，用于将 JSON 字符串或字符串数组反序列化为 `Option<Vec<String>>`
//!
//! 此模块提供了一种灵活的方式来处理字段可能为单个字符串或字符串数组的情况。
//! 它会将单个字符串转换为只有一个元素的向量，或将字符串数组转换为向量。

use serde::{
    de::{self, Visitor}, Deserializer, Serialize,
    Serializer,
};
use std::fmt;

/// # 将 `Option<Vec<String>>` 序列化为 JSON 数据
///
/// ## 支持的格式
/// - `None` -> `null`
/// - `Some(vec![])` -> `null`
/// - `Some(vec!["string"])` -> `"string"`
/// - `Some(vec!["string1", "string2"])` -> `["string1", "string2"]`
///
/// ## 示例
/// ```
/// use serde::Serialize;
/// #[derive(Serialize)]
/// struct Example {
pub fn serialize<S>(level: &Option<Vec<String>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match level {
        Some(vec) if vec.len() == 1 => serializer.serialize_str(&vec[0]),
        Some(vec) if !vec.is_empty() => vec.serialize(serializer),
        _ => serializer.serialize_none(),
    }
}

/// # 将 JSON 数据反序列化为 `Option<Vec<String>>` 类型
///
/// ## 支持的格式
/// - `null` -> `None`
/// - `"string"` -> `Some(vec!["string"])`
/// - `["string1", "string2"]` -> `Some(vec!["string1", "string2"])`
///
/// ## 示例
/// ```
/// use serde::Deserialize;
/// #[derive(Deserialize)]
/// struct Example {
///     #[serde(deserialize_with = "crate::serde::vec_option_serde::deserialize")]
///     tags: Option<Vec<String>>,
/// }
/// ```
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_option(OptionVisitor)
}

/// # 用于处理 Option 类型的访问器
struct OptionVisitor;

impl<'de> Visitor<'de> for OptionVisitor {
    type Value = Option<Vec<String>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string, comma-separated string, or array of strings")
    }

    /// 处理 None 值的情况
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    /// 处理 Some 值的情况，委托给 StringOrVecVisitor 处理具体类型
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 使用 deserialize_any 来处理不同的类型
        deserializer.deserialize_any(StringOrVecOptionVisitor)
    }
}

/// # 用于处理字符串或字符串数组的访问器
struct StringOrVecOptionVisitor;

impl<'de> Visitor<'de> for StringOrVecOptionVisitor {
    type Value = Option<Vec<String>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string, comma-separated string, or array of strings")
    }

    /// # 处理字符串引用（&str）类型
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(v.to_string())
    }

    /// # 处理字符串（String）类型
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // 将字符串通过逗号分割转换为 vec
        let vec: Vec<_> = v
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(Some(vec))
    }

    /// # 处理序列（数组）类型
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        // 逐个读取序列中的元素
        while let Some(element) = seq.next_element::<String>()? {
            let element = element.trim().to_string();
            if element.is_empty() {
                continue;
            }
            vec.push(element);
        }
        Ok(Some(vec))
    }
}
