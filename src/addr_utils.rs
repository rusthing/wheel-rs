//! # 地址工具模块
//!
//! 提供网络地址（[`Addr`]）的解析、格式化展示以及 serde 序列化支持。

use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::net::ToSocketAddrs;
use thiserror::Error;

/// # 地址解析错误
///
/// 当地址字符串无法解析为 [`Addr`] 时返回，包含具体的解析失败信息。
#[derive(Debug, Error)]
pub enum AddrError {
    #[error("Fail to parse Addr string: {0}")]
    Parse(String),
}

/// # 网络地址结构体
///
/// 表示由主机名或 IP 地址（`host`）与可选端口号（`port`）组成的网络地址。
/// 支持 IPv4、IPv6（带方括号形式）以及主机名等多种形式。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Addr {
    /// 主机名或 IP 地址；IPv6 地址**不带**方括号（方括号只在格式化时补上）
    pub host: String,
    /// 端口号；`None` 表示未指定端口
    pub port: Option<u16>,
}

impl Addr {
    /// # 创建新的地址
    ///
    /// ## 参数
    ///
    /// * `host` - 主机名或 IP 地址字符串
    /// * `port` - 可选的端口号，为 `None` 时表示不指定端口
    ///
    /// ## 返回值
    ///
    /// 返回包含指定主机与端口的 [`Addr`] 实例。
    pub fn new(host: String, port: Option<u16>) -> Addr {
        Addr { host, port }
    }

    /// # 从字符串解析地址
    ///
    /// 支持的格式：
    /// - `host`：仅主机名或 IP，无端口
    /// - `host:port`：主机加端口
    /// - `[ipv6]:port`：IPv6 地址用方括号包裹并带端口
    /// - 包含多个 `:` 的字符串按 IPv6 地址处理（无端口）
    ///
    /// ## 参数
    ///
    /// * `s` - 待解析的地址字符串
    ///
    /// ## 返回值
    ///
    /// 解析成功返回 [`Addr`] 实例。
    ///
    /// ## 错误
    ///
    /// 地址格式非法、主机为空或端口无法解析为 `u16` 时返回 [`AddrError::Parse`]。
    ///
    /// ## 示例
    ///
    /// ```
    /// use wheel_rs::addr_utils::Addr;
    ///
    /// let addr = Addr::from_str("example.com:8080").unwrap();
    /// assert_eq!(addr.host, "example.com");
    /// assert_eq!(addr.port, Some(8080));
    ///
    /// // IPv6 带方括号：方括号会被去掉，格式化时再补上
    /// let addr = Addr::from_str("[::1]:8080").unwrap();
    /// assert_eq!(addr.to_string(), "[::1]:8080");
    ///
    /// // 也可以走标准 trait
    /// assert!(("example.com:80").parse::<Addr>().is_ok());
    /// ```
    // 保留固有 `from_str`：下游大量以 `Addr::from_str(..)` 调用，不能改成只有 trait 方法
    // （那会要求调用方 `use std::str::FromStr`）。trait 实现见下方，两者共用 `parse`。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Addr, AddrError> {
        Self::parse(s)
    }

    /// 内部解析实现：`from_str`（固有方法）与 [`std::str::FromStr`] 共用
    fn parse(s: &str) -> Result<Addr, AddrError> {
        let (host, port) = if let Some(inner) = s.strip_prefix('[') {
            let bracket_end = inner
                .find(']')
                .ok_or_else(|| AddrError::Parse(format!("Invalid address: {s}")))?;
            let host = &inner[..bracket_end];
            let rest = &inner[bracket_end + 1..];
            let port = if let Some(port_str) = rest.strip_prefix(':') {
                Some(
                    port_str
                        .parse::<u16>()
                        .map_err(|_| AddrError::Parse(format!("Invalid port: {s}")))?,
                )
            } else if rest.is_empty() {
                None
            } else {
                return Err(AddrError::Parse(format!("Invalid address: {s}")));
            };
            (host.to_string(), port)
        } else if s.matches(':').count() > 1 {
            (s.to_string(), None)
        } else if let Some((host, port_str)) = s.split_once(':') {
            let port = port_str
                .parse::<u16>()
                .map_err(|_| AddrError::Parse(format!("Invalid port: {s}")))?;
            (host.to_string(), Some(port))
        } else {
            (s.to_string(), None)
        };

        if host.is_empty() {
            return Err(AddrError::Parse(format!("Invalid address: {s}")));
        }

        Ok(Addr { host, port })
    }
}

impl std::str::FromStr for Addr {
    type Err = AddrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Addr::parse(s)
    }
}

impl Display for Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(port) = self.port {
            if self.host.contains(':') {
                write!(f, "[{}]:{}", self.host, port)
            } else {
                write!(f, "{}:{}", self.host, port)
            }
        } else {
            write!(f, "{}", self.host)
        }
    }
}

impl Serialize for Addr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Addr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Addr::from_str(&s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

impl ToSocketAddrs for Addr {
    type Iter = std::vec::IntoIter<std::net::SocketAddr>;

    fn to_socket_addrs(&self) -> Result<Self::Iter, std::io::Error> {
        let port = self.port.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("missing port for host: {}", self.host),
            )
        })?;
        (self.host.as_str(), port).to_socket_addrs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_host_only() {
        let a = Addr::from_str("example.com").unwrap();
        assert_eq!((a.host.as_str(), a.port), ("example.com", None));
        let a = Addr::from_str("192.168.1.1").unwrap();
        assert_eq!((a.host.as_str(), a.port), ("192.168.1.1", None));
    }

    #[test]
    fn parses_host_with_port() {
        let a = Addr::from_str("example.com:8080").unwrap();
        assert_eq!((a.host.as_str(), a.port), ("example.com", Some(8080)));
    }

    #[test]
    fn parses_bracketed_ipv6() {
        let a = Addr::from_str("[::1]:8080").unwrap();
        assert_eq!((a.host.as_str(), a.port), ("::1", Some(8080)));
        assert_eq!(a.to_string(), "[::1]:8080");

        let a = Addr::from_str("[::1]").unwrap();
        assert_eq!((a.host.as_str(), a.port), ("::1", None));
        assert_eq!(a.to_string(), "::1");
    }

    #[test]
    fn bare_ipv6_has_no_port() {
        let a = Addr::from_str("fe80::1").unwrap();
        assert_eq!((a.host.as_str(), a.port), ("fe80::1", None));
    }

    #[test]
    fn rejects_invalid_input() {
        assert!(Addr::from_str("").is_err());
        assert!(Addr::from_str(":8080").is_err());
        assert!(Addr::from_str("host:not-a-port").is_err());
        assert!(Addr::from_str("host:70000").is_err());
        assert!(Addr::from_str("[::1").is_err());
    }

    #[test]
    fn round_trip_through_display() {
        for s in ["example.com", "example.com:8080", "[::1]:8080", "fe80::1"] {
            let a = Addr::from_str(s).unwrap();
            assert_eq!(a.to_string(), s);
        }
    }

    #[test]
    fn socket_addrs_requires_port() {
        use std::net::ToSocketAddrs;
        let no_port = Addr::from_str("127.0.0.1").unwrap();
        assert!(no_port.to_socket_addrs().is_err());

        let with_port = Addr::from_str("127.0.0.1:80").unwrap();
        let addrs: Vec<_> = with_port.to_socket_addrs().unwrap().collect();
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0].port(), 80);
    }
}
