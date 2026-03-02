//! # 自定义序列化/反序列化器，用于处理 `Option<PathBuf>` 类型的数据
//!
//! 此模块提供了一系列函数来处理可选路径缓冲区类型的序列化和反序列化，
//! 支持将字符串转换为 Option<PathBuf>，以及将 Option<PathBuf> 序列化为适当的 JSON 格式。
//!

use crate::serde::path_buf_serde::PathBufVisitor;
use serde::{de::Visitor, Deserializer, Serializer};
use std::fmt;
use std::path::PathBuf;

/// # 将 `Option<PathBuf>` 序列化为 JSON 数据
///
/// ## 支持的格式
/// - `None` -> `null`
/// - `Some(PathBuf::from("/path/to/file"))` -> `"/path/to/file"`
///
/// ## 示例
/// ```rust
/// use serde::Serialize;
/// use std::path::PathBuf;
///
/// #[derive(Serialize)]
/// struct Example {
/// #[serde(serialize_with = "crate::serde::path_buf_option_serde::serialize")]
/// path: Option<PathBuf>,
/// }
/// ```
pub fn serialize<S>(path: &Option<PathBuf>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match path {
        Some(p) => serializer.serialize_str(p.to_string_lossy().as_ref()),
        None => serializer.serialize_none(),
    }
}

/// # 将 JSON 数据反序列化为 `Option<PathBuf>` 类型
///
/// ## 支持的格式
/// - `null` -> `None`
/// - `"path/to/file"` -> `Some(PathBuf::from("path/to/file"))`
///
/// ## 示例
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use serde_json;
/// use std::path::PathBuf;
///
/// #[derive(Serialize, Deserialize)]
/// struct Example {
/// #[serde(
/// serialize_with = "crate::serde::path_buf_option_serde::serialize",
/// deserialize_with = "crate::serde::path_buf_option_serde::deserialize"
/// )]
/// path: Option<PathBuf>,
/// }
///
/// let json = r#"{"path": "/home/user/file.txt"}"#;
/// let example: Example = serde_json::from_str(json).unwrap();
/// assert_eq!(example.path, Some(PathBuf::from("/home/user/file.txt")));
/// ```
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_option(OptionPathBufVisitor)
}

/// # 用于处理 Option<PathBuf> 类型的访问器
struct OptionPathBufVisitor;

impl<'de> Visitor<'de> for OptionPathBufVisitor {
    type Value = Option<PathBuf>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representing a file path or null")
    }

    /// 处理 None 值的情况
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    /// 处理 Some 值的情况
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PathBufVisitor).map(Some)
    }
}
