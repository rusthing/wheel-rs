//! # URN 工具模块
//!
//! 提供统一资源名称（Uniform Resource Name）相关的工具函数和数据结构。
//!
//! 该模块定义了 HTTP 方法枚举和 URN 结构体，用于解析和表示资源定位信息。
//! ```rust
//! use wheel_rs::urn_utils::{Urn, Method};
//!
//! let urn = Urn::new("GET:example.com".to_string());
//! assert!(matches!(urn.method, Method::Get));
//! assert_eq!(urn.url, "example.com");
//! ```
//! ## 示例
//! ```rust
//!
//! use wheel_rs::urn_utils::{Urn, Method};
//!
//! let urn = Urn::new("GET:example.com".to_string());
//! assert!(matches!(urn.method, Method::Get));
//! assert_eq!(urn.url, "example.com");
//! ```
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MethodError {
    #[error("Fail to parse method string: {0}")]
    Parse(String),
}

/// # HTTP 方法枚举
///
/// 定义了常用的 HTTP 方法类型，包括 GET、POST、PUT 和 DELETE
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Method {
    /// GET 方法 - 用于请求获取资源
    Get,
    /// POST 方法 - 用于提交数据到服务器
    Post,
    /// PUT 方法 - 用于更新或创建资源
    Put,
    /// DELETE 方法 - 用于删除资源
    Delete,
}

impl<'de> Deserialize<'de> for Method {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_uppercase();
        match s.as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            _ => Err(serde::de::Error::custom(format!("Invalid method: {}", s))),
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Method::Get => "GET".to_string(),
                Method::Post => "POST".to_string(),
                Method::Put => "PUT".to_string(),
                Method::Delete => "DELETE".to_string(),
            }
        )
    }
}

impl Method {
    pub fn from_str(method: &str) -> Result<Self, MethodError> {
        match method.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            _ => Err(MethodError::Parse(format!("Invalid method: {}", method))),
        }
    }
}

#[derive(Debug, Error)]
pub enum UrnError {
    #[error("Fail to parse Urn string: {0}")]
    Parse(String),
    #[error("{0}")]
    InvalidMethod(#[from] MethodError),
}

/// # URN 结构体
///
/// 用于表示统一资源名称（Uniform Resource Name），包含方法和 URL 两部分
/// 支持两种格式：
/// 1. 显式指定方法：`GET:example.com`
/// 2. HTTP/HTTPS 前缀：`http:example.com` 或 `https:example.com`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Urn {
    /// HTTP 方法
    pub method: Method,
    /// 资源 URL
    pub url: String,
}

impl Serialize for Urn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Urn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Urn::from_str(&s).map_err(|e| serde::de::Error::custom(format!("{e}")))
    }
}

impl std::fmt::Display for Urn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.method.to_string(), self.url)
    }
}

impl Urn {
    pub fn new(method: String, url: String) -> Result<Self, UrnError> {
        Ok(Self {
            method: Method::from_str(&method)?,
            url,
        })
    }

    /// # 创建一个新的 URN 实例
    ///
    /// ## 参数
    ///
    /// * `urn` - 表示 URN 的字符串，支持两种格式：
    ///   - 显式方法格式：`METHOD:URL`，例如 `GET:example.com`
    ///   - HTTP 前缀格式：`http:URL` 或 `https:URL`，会自动设置方法为 GET
    ///
    /// ## 返回值
    ///
    /// 返回解析后的 URN 实例
    ///
    /// ## Panics
    ///
    /// * 当 URL 部分为空时会触发 panic
    /// * 当方法部分不是有效的 HTTP 方法时会触发 panic
    ///
    /// ## 示例
    ///
    /// ```
    /// use wheel_rs::urn_utils::Urn;
    ///
    /// let urn = Urn::new("GET:example.com".to_string());
    /// assert_eq!(urn.method.to_string(), "GET");
    /// assert_eq!(urn.url, "example.com");
    ///
    /// let urn = Urn::new("http:example.com".to_string());
    /// assert_eq!(urn.method.to_string(), "GET");
    /// assert_eq!(urn.url, "http:example.com");
    /// ```
    pub fn from_str(urn: &str) -> Result<Self, UrnError> {
        if urn.starts_with("http:") || urn.starts_with("https:") {
            return Ok(Self {
                method: Method::Get,
                url: urn.to_string(),
            });
        }

        // 按 ':' 分割URN获取method和url
        let parts: Vec<&str> = urn.splitn(2, ':').collect();
        let (method, url) = match parts.len() {
            1 => ("GET", parts[0].trim()),
            2 => (parts[0].trim(), parts[1].trim()),
            _ => Err(UrnError::Parse(format!("Invalid URN \"{urn}\"")))?,
        };

        if url.is_empty() {
            Err(UrnError::Parse(format!("Invalid URN \"{urn}\"")))?
        }

        Ok(Self {
            method: match method.to_uppercase().as_str() {
                "GET" => Method::Get,
                "POST" => Method::Post,
                "PUT" => Method::Put,
                "DELETE" => Method::Delete,
                _ => panic!("Invalid method: {}", method),
            },
            url: url.to_string(),
        })
    }

    /// 比较 Urn 与给定的 method 和 url 是否相等
    ///
    /// # 参数
    ///
    /// * `method` - HTTP 方法字符串
    /// * `url` - URL 字符串
    ///
    /// # 返回值
    ///
    /// 如果 method 和 url 都与 Urn 的对应字段相等则返回 true，否则返回 false
    ///
    /// # 示例
    ///
    /// ```rust
    /// use wheel_rs::urn_utils::Urn;
    ///
    /// let urn = Urn::new("GET:example.com".to_string());
    /// assert!(urn.matches("GET", "example.com"));
    /// assert!(!urn.matches("POST", "example.com"));
    /// ```
    pub fn matches(&self, method: &str, url: &str) -> bool {
        self.method.to_string() == method.to_uppercase() && url.starts_with(&self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urn_new() {
        let urn = Urn::from_str("GET:example.com").unwrap();
        assert!(matches!(urn.method, Method::Get));
        assert_eq!(urn.url, "example.com");
    }

    #[test]
    fn test_urn_with_complex_url() {
        let urn = Urn::from_str("POST:api.example.com/v1/users").unwrap();
        assert!(matches!(urn.method, Method::Post));
        assert_eq!(urn.url, "api.example.com/v1/users");
    }

    #[test]
    #[should_panic(expected = "Invalid URN:")]
    fn test_urn_with_empty_url() {
        let _urn = Urn::from_str("PUT:");
    }

    #[test]
    fn test_http_prefix_urls() {
        let urn = Urn::from_str("http:example.com").unwrap();
        assert!(matches!(urn.method, Method::Get));
        assert_eq!(urn.url, "http:example.com");

        let urn = Urn::from_str("https:example.com").unwrap();
        assert!(matches!(urn.method, Method::Get));
        assert_eq!(urn.url, "https:example.com");
    }
}
