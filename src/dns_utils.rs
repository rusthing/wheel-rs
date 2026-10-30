//! # DNS 工具模块
//!
//! 提供 DNS 解析相关的实用工具函数。
//!
//! ## 示例
//!
//! ```
//! use wheel_rs::dns_utils::parse_host;
//!
//! let ip = parse_host("localhost").unwrap();
//! println!("IP 地址: {}", ip);
//! ```

use dns_lookup::lookup_host;
use std::io;
use std::net::IpAddr;
use thiserror::Error;
use tracing::info;

/// # DNS 工具错误类型
///
/// 统一本模块的失败情形，便于调用方 `match` 具体原因，而不是解析错误字符串。
#[derive(Error, Debug)]
pub enum DnsError {
    /// 地址整体格式非法，例如 `[::1` 缺少右方括号
    #[error("invalid address: {0}")]
    InvalidAddress(String),

    /// 端口部分无法解析为 `u16`
    #[error("invalid port in address: {0}")]
    InvalidPort(String),

    /// 系统 DNS 解析失败（底层 `getaddrinfo` 出错）
    #[error("DNS lookup failed for host `{host}`")]
    Lookup {
        /// 被解析的主机名
        host: String,
        /// 底层 IO 错误
        #[source]
        source: io::Error,
    },

    /// DNS 解析成功，但没有返回任何地址
    #[error("no address found for host `{host}`")]
    NoAddress {
        /// 被解析的主机名
        host: String,
    },
}

/// # 解析主机的字符串成 IP 地址
///
/// 尝试将主机名或 IP 地址字符串解析为 [`IpAddr`] 类型。
/// 首先尝试直接解析为 IP 地址，如果失败则进行 DNS 解析。
///
/// ## 参数
///
/// * `host` - 一个字符串切片，表示要解析的主机名或 IP 地址
///
/// ## 返回值
///
/// 解析成功返回 [`IpAddr`]。
///
/// ## 错误
///
/// * 系统 DNS 解析失败时返回 [`DnsError::Lookup`]
/// * 解析结果为空时返回 [`DnsError::NoAddress`]
///
/// ## 示例
///
/// ```
/// use wheel_rs::dns_utils::parse_host;
///
/// let ip = parse_host("127.0.0.1").unwrap();
/// assert_eq!(ip.to_string(), "127.0.0.1");
/// ```
pub fn parse_host(host: &str) -> Result<IpAddr, DnsError> {
    // 尝试直接解析为 IP 地址
    if let Ok(ip_addr) = host.parse::<IpAddr>() {
        return Ok(ip_addr);
    }

    // 不是 IP，尝试 DNS 解析。
    // `dns_lookup::LookupError` 实现了 `Into<io::Error>`，转成 io::Error 可以避免把
    // 第三方错误类型暴露在本库的公开 API 上。
    let mut addrs = lookup_host(host).map_err(|e| DnsError::Lookup {
        host: host.to_string(),
        source: e.into(),
    })?;
    let ip_addr = addrs.next().ok_or_else(|| DnsError::NoAddress {
        host: host.to_string(),
    })?;
    info!("Resolved hostname: {host} -> {ip_addr}");
    Ok(ip_addr)
}

/// # 解析主机名和端口号字符串为 IP 地址和端口号
///
/// 支持多种格式的主机名和端口号字符串解析：
/// - `"192.168.1.1:8080"`
/// - `"example.com:8080"`
/// - `"[::1]:8080"`（IPv6 带端口，方括号必需）
/// - `"192.168.1.1"`（无端口号，默认端口为 0）
/// - `"example.com"`（无端口号，默认端口为 0）
/// - `"::1"`（无端口的裸 IPv6，默认端口为 0）
///
/// ## 参数
///
/// * `host_port` - 包含主机名和可选端口号的字符串
///
/// ## 返回值
///
/// 解析成功返回 `(IpAddr, u16)` 元组；未提供端口号时端口默认为 `0`。
///
/// ## 错误
///
/// * [`DnsError::InvalidAddress`] - 地址整体格式非法（如 `[::1` 缺右括号）
/// * [`DnsError::InvalidPort`] - 端口部分不是合法的 `u16`
/// * [`DnsError::Lookup`] / [`DnsError::NoAddress`] - 主机名解析失败
///
/// ## 示例
///
/// ```
/// use wheel_rs::dns_utils::parse_host_port;
///
/// let (ip, port) = parse_host_port("localhost:8080").unwrap();
/// assert_eq!(port, 8080);
///
/// // 无端口的裸 IPv6 也能解析
/// let (ip, port) = parse_host_port("::1").unwrap();
/// assert_eq!(port, 0);
/// assert!(ip.is_ipv6());
/// ```
pub fn parse_host_port(host_port: &str) -> Result<(IpAddr, u16), DnsError> {
    let host_port = host_port.trim();

    // 处理 IPv6 地址带端口的情况 [::1]:8080
    if let Some(rest) = host_port.strip_prefix('[') {
        if let Some((host, port_str)) = rest.split_once("]:") {
            let port = port_str
                .parse::<u16>()
                .map_err(|_| DnsError::InvalidPort(host_port.to_string()))?;
            return Ok((parse_host(host)?, port));
        }
        // 只有方括号、没有端口：[::1]
        if let Some(host) = rest.strip_suffix(']') {
            return Ok((parse_host(host)?, 0));
        }
        return Err(DnsError::InvalidAddress(host_port.to_string()));
    }

    // 处理一般的 host:port 格式。
    // 注意：裸 IPv6 地址本身就含多个冒号（如 "::1"、"fe80::1"），
    // 因此只有当 `:` 之前的部分不含冒号时，才把它当作 host:port 处理。
    if let Some((host, port_str)) = host_port.rsplit_once(':')
        && !host.contains(':')
    {
        let port = port_str
            .parse::<u16>()
            .map_err(|_| DnsError::InvalidPort(host_port.to_string()))?;
        return Ok((parse_host(host)?, port));
    }

    // 没有找到端口号（或整体是裸 IPv6），只解析主机名，默认端口为 0
    let ip_addr = parse_host(host_port)?;
    Ok((ip_addr, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipv4_with_port() {
        let (ip, port) = parse_host_port("192.168.1.1:8080").unwrap();
        assert_eq!(ip.to_string(), "192.168.1.1");
        assert_eq!(port, 8080);
    }

    #[test]
    fn ipv4_without_port() {
        let (ip, port) = parse_host_port("192.168.1.1").unwrap();
        assert_eq!(ip.to_string(), "192.168.1.1");
        assert_eq!(port, 0);
    }

    #[test]
    fn ipv6_with_brackets_and_port() {
        let (ip, port) = parse_host_port("[::1]:8080").unwrap();
        assert!(ip.is_ipv6());
        assert_eq!(port, 8080);
    }

    #[test]
    fn ipv6_bare() {
        for s in ["::1", "[::1]", "fe80::1"] {
            let (ip, port) = parse_host_port(s).unwrap();
            assert!(ip.is_ipv6(), "{s} 应解析为 IPv6");
            assert_eq!(port, 0, "{s} 的端口应为默认值 0");
        }
    }

    #[test]
    fn invalid_port_is_error() {
        assert!(parse_host_port("example.com:not-a-port").is_err());
        assert!(parse_host_port("[::1]:not-a-port").is_err());
    }

    #[test]
    fn failures_map_to_typed_errors() {
        // 缺右方括号 → 整体格式非法
        assert!(matches!(
            parse_host_port("[::1"),
            Err(DnsError::InvalidAddress(_))
        ));
        // 端口超出 u16 范围
        assert!(matches!(
            parse_host_port("example.com:70000"),
            Err(DnsError::InvalidPort(_))
        ));
        assert!(matches!(
            parse_host_port("[::1]:70000"),
            Err(DnsError::InvalidPort(_))
        ));
    }
}
