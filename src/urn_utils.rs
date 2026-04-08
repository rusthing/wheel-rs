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
use serde::{Deserialize, Deserializer};

/// # HTTP 方法枚举
///
/// 定义了常用的 HTTP 方法类型，包括 GET、POST、PUT 和 DELETE
#[derive(Debug, Clone, PartialEq)]
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

impl Method {
    /// 将 Method 枚举转换为对应的字符串表示
    ///
    /// # Returns
    ///
    /// 返回 HTTP 方法的大写字符串表示形式
    ///
    /// # Examples
    ///
    /// ```
    /// use wheel_rs::urn_utils::Method;
    ///
    /// let method = Method::Get;
    /// assert_eq!(method.to_string(), "GET");
    /// ```
    pub fn to_string(&self) -> String {
        match self {
            Method::Get => "GET".to_string(),
            Method::Post => "POST".to_string(),
            Method::Put => "PUT".to_string(),
            Method::Delete => "DELETE".to_string(),
        }
    }
}

/// # URN 结构体
///
/// 用于表示统一资源名称（Uniform Resource Name），包含方法和 URL 两部分
/// 支持两种格式：
/// 1. 显式指定方法：`GET:example.com`
/// 2. HTTP/HTTPS 前缀：`http:example.com` 或 `https:example.com`
#[derive(Debug, Clone, PartialEq)]
pub struct Urn {
    /// HTTP 方法
    pub method: Method,
    /// 资源 URL
    pub url: String,
}

impl<'de> Deserialize<'de> for Urn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Urn::new(s))
    }
}

impl Urn {
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
    pub fn new(urn: String) -> Self {
        if urn.starts_with("http:") || urn.starts_with("https:") {
            return Self {
                method: Method::Get,
                url: urn,
            };
        }

        // 按 ':' 分割URN获取method和url
        let parts: Vec<&str> = urn.splitn(2, ':').collect();
        let method = parts[0].trim();
        let url = parts[1].trim();
        if url.is_empty() {
            panic!("Invalid URN: {}", urn);
        }

        Self {
            method: match method.to_uppercase().as_str() {
                "GET" => Method::Get,
                "POST" => Method::Post,
                "PUT" => Method::Put,
                "DELETE" => Method::Delete,
                _ => panic!("Invalid method: {}", method),
            },
            url: url.to_string(),
        }
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
        let urn = Urn::new("GET:example.com".to_string());
        assert!(matches!(urn.method, Method::Get));
        assert_eq!(urn.url, "example.com");
    }

    #[test]
    fn test_urn_with_complex_url() {
        let urn = Urn::new("POST:api.example.com/v1/users".to_string());
        assert!(matches!(urn.method, Method::Post));
        assert_eq!(urn.url, "api.example.com/v1/users");
    }

    #[test]
    #[should_panic(expected = "Invalid URN:")]
    fn test_urn_with_empty_url() {
        let _urn = Urn::new("PUT:".to_string());
    }

    #[test]
    fn test_http_prefix_urls() {
        let urn = Urn::new("http:example.com".to_string());
        assert!(matches!(urn.method, Method::Get));
        assert_eq!(urn.url, "http:example.com");

        let urn = Urn::new("https:example.com".to_string());
        assert!(matches!(urn.method, Method::Get));
        assert_eq!(urn.url, "https:example.com");
    }
}
