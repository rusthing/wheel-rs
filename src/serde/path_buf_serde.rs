//! # 自定义序列化/反序列化器，用于处理 `PathBuf` 类型的数据
//!
//! 此模块提供了一系列函数来处理路径缓冲区类型的序列化和反序列化，
//! 支持将字符串转换为 PathBuf，以及将 PathBuf 序列化为适当的 JSON 格式。
//!

use serde::{de::Visitor, Deserializer, Serializer};
use std::fmt;
use std::path::PathBuf;

/// # 将 `PathBuf` 序列化为 JSON 字符串
///
/// ## 支持的格式
/// - `PathBuf::from("/path/to/file")` -> `"/path/to/file"`
/// - `PathBuf::from("relative/path")` -> `"relative/path"`
///
/// ## 示例
/// ```rust
/// use serde::Serialize;
/// use std::path::PathBuf;
///
/// #[derive(Serialize)]
/// struct Example {
/// #[serde(serialize_with = "crate::serde::path_buf_serde::serialize")]
/// path: PathBuf,
/// }
/// ```
pub fn serialize<S>(path: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(path.to_string_lossy().as_ref())
}

/// # 将 JSON 字符串反序列化为 `PathBuf` 类型
///
/// ## 支持的格式
/// - `"path/to/file"` -> `PathBuf::from("path/to/file")`
/// - `"/absolute/path"` -> `PathBuf::from("/absolute/path")`
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
/// serialize_with = "crate::serde::path_buf_serde::serialize",
/// deserialize_with = "crate::serde::path_buf_serde::deserialize"
/// )]
/// path: PathBuf,
/// }
///
/// let json = r#"{"path": "/home/user/file.txt"}"#;
/// let example: Example = serde_json::from_str(json).unwrap();
/// assert_eq!(example.path, PathBuf::from("/home/user/file.txt"));
/// ```
pub fn deserialize<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(PathBufVisitor)
}

/// # 用于处理字符串到 PathBuf 的访问器
pub(crate) struct PathBufVisitor;

impl<'de> Visitor<'de> for PathBufVisitor {
    type Value = PathBuf;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representing a file path")
    }

    /// # 处理字符串引用（&str）类型
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PathBuf::from(value))
    }

    /// # 处理字符串（String）类型
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PathBuf::from(value))
    }
}
