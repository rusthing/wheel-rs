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

impl Method {
    /// 将 Method 枚举转换为对应的字符串表示
    ///
    /// # Returns
    ///
    /// 返回 HTTP 方法的大写字符串表示形式
    ///
    /// # Examples
    ///
    ///
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
    /// let urn = Urn::new("GET:example.com".to_string());
    /// assert!(matches!(urn.method, Method::Get));
    /// assert_eq!(urn.url, "example.com");
    ///
    /// let urn = Urn::new("http:example.com".to_string());
    /// assert!(matches!(urn.method, Method::Get));
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
