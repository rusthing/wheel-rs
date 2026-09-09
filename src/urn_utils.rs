//! # URN 工具模块
//!
//! 提供统一资源名称（Uniform Resource Name）相关的工具函数和数据结构。
//!
//! 该模块定义了 HTTP 方法枚举 [`Method`] 和 URN 结构体 [`Urn`]，
//! 用于解析和表示"方法 + 资源路径"形式的接口定位信息。
//!
//! ## 示例
//!
//! ```
//! use wheel_rs::urn_utils::{Method, Urn};
//!
//! let urn = Urn::from_str("GET:example.com").unwrap();
//! assert!(matches!(urn.method, Some(Method::Get)));
//! assert_eq!(urn.url, "example.com");
//! ```
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// # HTTP 方法解析错误
#[derive(Debug, Error)]
pub enum MethodError {
    /// 方法字符串不在受支持的范围内（GET/POST/PUT/DELETE/OPTIONS/HEAD/PATCH）
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
    /// OPTIONS 方法 - 用于获取资源的可用方法
    Options,
    /// HEAD 方法 - 用于请求获取资源的header头，不返回Body
    Head,
    /// PATCH 方法 - 用于部分更新资源的指定字段
    Patch,
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
            "OPTIONS" => Ok(Method::Options),
            "HEAD" => Ok(Method::Head),
            "PATCH" => Ok(Method::Patch),
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
                Method::Options => "OPTIONS".to_string(),
                Method::Head => "HEAD".to_string(),
                Method::Patch => "PATCH".to_string(),
            }
        )
    }
}

impl Method {
    /// # 从字符串解析 HTTP 方法
    ///
    /// 大小写不敏感，受支持的方法为 GET、POST、PUT、DELETE、OPTIONS、HEAD、PATCH。
    ///
    /// ## 返回值
    ///
    /// * `Ok(Method)` - 解析成功
    /// * `Err(MethodError::Parse)` - 字符串不在受支持范围内
    ///
    /// ## 示例
    ///
    /// ```
    /// use wheel_rs::urn_utils::{Method, MethodError};
    ///
    /// assert!(matches!(Method::from_str("get").unwrap(), Method::Get));
    /// assert!(matches!(Method::from_str("PATCH").unwrap(), Method::Patch));
    /// assert!(Method::from_str("TRACE").is_err());
    /// ```
    pub fn from_str(method: &str) -> Result<Self, MethodError> {
        match method.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "OPTIONS" => Ok(Self::Options),
            "HEAD" => Ok(Self::Head),
            "PATCH" => Ok(Self::Patch),
            _ => Err(MethodError::Parse(format!("Invalid method: {}", method))),
        }
    }
}

/// # URN 解析错误
#[derive(Debug, Error)]
pub enum UrnError {
    /// URN 字符串格式非法，例如 URL 部分为空
    #[error("Fail to parse Urn string: {0}")]
    Parse(String),
    /// URN 中的方法部分无法解析为受支持的 HTTP 方法
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
    pub method: Option<Method>,
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
        if let Some(method) = &self.method {
            write!(f, "{}:{}", method.to_string(), self.url)
        } else {
            write!(f, "{}", self.url)
        }
    }
}

impl Urn {
    /// # 由方法与 URL 直接构造 URN
    ///
    /// 与 [`Urn::from_str`] 不同，此函数不做字符串解析，而是分别接收方法与 URL。
    ///
    /// ## 参数
    ///
    /// * `method` - HTTP 方法字符串（如 `"GET"`），大小写不敏感；传 `None` 表示不限定方法
    /// * `url` - 资源 URL
    ///
    /// ## 返回值
    ///
    /// * `Ok(Urn)` - 构造成功
    /// * `Err(UrnError::InvalidMethod)` - `method` 不是受支持的 HTTP 方法
    ///
    /// ## 示例
    ///
    /// ```
    /// use wheel_rs::urn_utils::{Method, Urn};
    ///
    /// let urn = Urn::new(Some("get".to_string()), "example.com".to_string()).unwrap();
    /// assert!(matches!(urn.method, Some(Method::Get)));
    /// assert_eq!(urn.url, "example.com");
    ///
    /// // 不限定方法
    /// let urn = Urn::new(None, "example.com".to_string()).unwrap();
    /// assert!(urn.method.is_none());
    /// ```
    pub fn new(method: Option<String>, url: String) -> Result<Self, UrnError> {
        let method = method.map(|m| Method::from_str(&m)).transpose()?;
        Ok(Self { method, url })
    }

    /// # 从字符串解析 URN
    ///
    /// 支持三种输入格式：
    /// 1. 显式方法：`METHOD:URL`，例如 `GET:example.com`
    /// 2. HTTP 前缀：`http:...` 或 `https:...`，方法自动置为 `GET`，且**整个输入原样作为 url 保留**（不剥离前缀）
    /// 3. 无方法：不含 `:` 时，方法为 `None`，整个输入作为 url
    ///
    /// ## 参数
    ///
    /// * `urn` - 待解析的 URN 字符串，方法与 URL 两侧空白会被 trim
    ///
    /// ## 返回值
    ///
    /// * `Ok(Urn)` - 解析成功
    /// * `Err(UrnError::Parse)` - URL 部分为空，例如 `"PUT:"`
    /// * `Err(UrnError::InvalidMethod)` - 方法不是受支持的 HTTP 方法
    ///
    /// 解析失败一律返回 `Err`，**不会 panic**。
    ///
    /// ## 示例
    ///
    /// ```
    /// use wheel_rs::urn_utils::{Method, Urn};
    ///
    /// let urn = Urn::from_str("GET:example.com").unwrap();
    /// assert!(matches!(urn.method, Some(Method::Get)));
    /// assert_eq!(urn.url, "example.com");
    ///
    /// // http/https 前缀：方法推断为 GET，url 保留完整输入
    /// let urn = Urn::from_str("http:example.com").unwrap();
    /// assert!(matches!(urn.method, Some(Method::Get)));
    /// assert_eq!(urn.url, "http:example.com");
    ///
    /// // URL 为空时返回 Err 而非 panic
    /// assert!(Urn::from_str("PUT:").is_err());
    /// ```
    pub fn from_str(urn: &str) -> Result<Self, UrnError> {
        if urn.starts_with("http:") || urn.starts_with("https:") {
            return Ok(Self {
                method: Some(Method::Get),
                url: urn.to_string(),
            });
        }

        // 按 ':' 分割URN获取method和url
        let parts: Vec<&str> = urn.splitn(2, ':').collect();
        let (method, url) = match parts.len() {
            1 => (None, parts[0].trim()),
            2 => (Some(parts[0].trim()), parts[1].trim()),
            _ => Err(UrnError::Parse(format!("Invalid URN \"{urn}\"")))?,
        };

        if url.is_empty() {
            Err(UrnError::Parse(format!("Invalid URN \"{urn}\"")))?
        }

        let method = method.map(|m| Method::from_str(&m)).transpose()?;

        Ok(Self {
            method,
            url: url.to_string(),
        })
    }

    /// # 判断 URN 是否匹配给定的方法与 URL
    ///
    /// 匹配规则：
    /// - **方法**：若自身 `method` 为 `Some`，则须与传入 `method`（大写化后）相等；
    ///   若自身为 `None`，则**不限定方法**，任何方法都视为匹配。
    /// - **URL**：采用**前缀匹配**（`url.starts_with(&self.url)`），而非相等比较。
    ///   因此 `self.url` 为 `"example.com"` 时可匹配 `"example.com/api/users"`。
    ///
    /// ## 参数
    ///
    /// * `method` - HTTP 方法字符串，大小写不敏感
    /// * `url` - 待匹配的实际请求 URL
    ///
    /// ## 返回值
    ///
    /// 方法与 URL 前缀均匹配时返回 `true`，否则返回 `false`。
    ///
    /// ## 示例
    ///
    /// ```
    /// use wheel_rs::urn_utils::Urn;
    ///
    /// let urn = Urn::from_str("GET:example.com").unwrap();
    /// assert!(urn.matches("GET", "example.com"));
    /// // URL 为前缀匹配
    /// assert!(urn.matches("get", "example.com/api/users"));
    /// assert!(!urn.matches("POST", "example.com"));
    ///
    /// // 未指定方法时不限定方法
    /// let any = Urn::from_str("example.com").unwrap();
    /// assert!(any.matches("POST", "example.com"));
    /// ```
    pub fn matches(&self, method: &str, url: &str) -> bool {
        if let Some(self_method) = &self.method {
            if self_method.to_string() != method.to_uppercase() {
                return false;
            }
        }
        url.starts_with(&self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urn_new() {
        let urn = Urn::from_str("GET:example.com").unwrap();
        assert!(matches!(urn.method, Some(Method::Get)));
        assert_eq!(urn.url, "example.com");
    }

    #[test]
    fn test_urn_with_complex_url() {
        let urn = Urn::from_str("POST:api.example.com/v1/users").unwrap();
        assert!(matches!(urn.method, Some(Method::Post)));
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
        assert!(matches!(urn.method, Some(Method::Get)));
        assert_eq!(urn.url, "http:example.com");

        let urn = Urn::from_str("https:example.com").unwrap();
        assert!(matches!(urn.method, Some(Method::Get)));
        assert_eq!(urn.url, "https:example.com");
    }
}
