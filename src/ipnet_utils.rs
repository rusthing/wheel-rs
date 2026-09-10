//! # IP 网络工具模块
//!
//! 提供 IP 网络（`ipnet::IpNet`）的比较排序与精确性判断工具。

use ipnet::IpNet;
use std::cmp::Ordering;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IpnetError {
    #[error("no local ip address found")]
    NoLocalIp,
}

/// # 比较两个网络
///
/// 精确匹配的网络（IPv4 前缀长度为 32 或 IPv6 前缀长度为 128）会排在非精确网络之前；
/// 其余情况按 `IpNet` 的默认顺序（[`IpNet::cmp`]）比较。
///
/// ## 返回值
///
/// 返回 `a` 与 `b` 之间的 [`Ordering`]，用于排序场景。
pub fn com_ip(a: &IpNet, b: &IpNet) -> Ordering {
    match (is_exact(a), is_exact(b)) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => a.cmp(b),
    }
}

/// # 判断网络是否为精确地址
///
/// 判断网络是否精确到单个 IP 地址，即 IPv4 前缀长度为 32 或 IPv6 前缀长度为 128。
///
/// ## 返回值
///
/// 若网络为精确地址则返回 `true`，否则返回 `false`。
pub fn is_exact(net: &IpNet) -> bool {
    match net {
        IpNet::V4(n) => n.prefix_len() == 32,
        IpNet::V6(n) => n.prefix_len() == 128,
    }
}

/// 获取本机首个非 loopback 的 IPv4 地址。
///
/// 通过遍历系统网卡接口获取，不依赖外网连通性。
///
/// ## 参数
/// * `sub_net` - 可选的子网掩码，若传入则仅返回匹配该子网的 IP。
pub fn get_local_ip(sub_net: Option<IpNet>) -> Result<String, IpnetError> {
    use nix::ifaddrs::getifaddrs;
    let addrs = getifaddrs().map_err(|_| IpnetError::NoLocalIp)?;
    for addr in addrs {
        if let Some(sockaddr) = addr.address {
            if let Some(sin) = sockaddr.as_sockaddr_in() {
                let ip = sin.ip().to_string();
                if ip == "127.0.0.1" {
                    continue;
                }
                if let Some(ref net) = sub_net {
                    let ip_addr: std::net::IpAddr = sin.ip().into();
                    if !net.contains(&ip_addr) {
                        continue;
                    }
                }
                return Ok(ip);
            }
        }
    }
    Err(IpnetError::NoLocalIp)
}
