//! # 自定义序列化/反序列化器，用于处理 `Vec<T>` 类型的数据
//!
//! 此模块提供了一系列函数来处理向量类型的序列化和反序列化，
//! 支持将字符串或字符串数组转换为向量，以及将向量序列化为适当的 JSON 格式。

use serde::{
    de::{self, Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
};
use std::fmt;

/// # 将 `Vec<String>` 序列化为 JSON 数组
///
/// ## 支持的格式
/// - `vec![]` -> `[]`
/// - `vec!["string"]` -> `["string"]`
/// - `vec!["string1", "string2"]` -> `["string1", "string2"]`
///
/// ## 示例
/// ```
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Example {
///     #[serde(serialize_with = "wheel_rs::serde::vec_serde::serialize")]
///     tags: Vec<String>,
/// }
/// ```
pub fn serialize<S>(vec: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(vec.len()))?;
    for element in vec {
        seq.serialize_element(element)?;
    }
    seq.end()
}

/// # 将 JSON 数据反序列化为 `Vec<String>` 类型
///
/// ## 支持的格式
/// - `["string1", "string2"]` -> `vec!["string1", "string2"]`（数组）
/// - `"a,b,c"` -> `vec!["a", "b", "c"]`（逗号分隔字符串，各项会 trim）
/// - `"single"` -> `vec!["single"]`（无逗号的字符串按单元素处理）
///
/// 数组元素与逗号分隔项中的**空字符串会被过滤掉**。
///
/// ## 不支持
///
/// `null` **不会**被转换为空数组，而是返回类型错误（`invalid type: null`）。
/// 若字段可能缺省，请改用 [`crate::serde::vec_option_serde`]。
///
/// ## 示例
/// ```
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Example {
///     #[serde(
///         serialize_with = "wheel_rs::serde::vec_serde::serialize",
///         deserialize_with = "wheel_rs::serde::vec_serde::deserialize"
///     )]
///     tags: Vec<String>,
/// }
///
/// // 数组形式
/// let example: Example = serde_json::from_str(r#"{"tags": ["tag1", "tag2"]}"#).unwrap();
/// assert_eq!(example.tags, vec!["tag1", "tag2"]);
///
/// // 逗号分隔字符串形式
/// let example: Example = serde_json::from_str(r#"{"tags": "a, b ,c"}"#).unwrap();
/// assert_eq!(example.tags, vec!["a", "b", "c"]);
/// ```
pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(StringOrVecVisitor)
}

/// # 用于处理字符串或字符串数组的访问器
struct StringOrVecVisitor;

impl<'de> Visitor<'de> for StringOrVecVisitor {
    type Value = Vec<String>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string, comma-separated string, or array of strings")
    }

    /// # 处理字符串引用（&str）类型
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(value.to_string())
    }

    /// # 处理字符串（String）类型
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // 将字符串通过逗号分割转换为 vec
        Ok(value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    /// # 处理序列（数组）类型
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
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
        Ok(vec)
    }
}
