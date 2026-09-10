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
#[derive(Debug, Clone)]
pub struct Addr {
    pub host: String,
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
    pub fn from_str(s: &str) -> Result<Addr, AddrError> {
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
        Addr::from_str(&s).map_err(|e| serde::de::Error::custom(format!("{:?}", e)))
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
