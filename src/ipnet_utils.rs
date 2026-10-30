//! # IP 网络工具模块
//!
//! 提供 IP 网络（`ipnet::IpNet`）的比较排序与精确性判断工具。

use ipnet::IpNet;
use std::cmp::Ordering;
use thiserror::Error;

/// # IP 网络工具错误类型
///
/// 定义 IP 网络工具模块中可能出现的错误。
#[derive(Error, Debug)]
pub enum IpnetError {
    /// 未找到本地 IP 地址
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

/// # 获取本机首个非 loopback 的 IPv4 地址。
///
/// 通过遍历系统网卡接口获取，不依赖外网连通性。
///
/// ## 参数
/// * `sub_net` - 可选的子网掩码，若传入则仅返回匹配该子网的 IP。
pub fn get_local_ip(sub_net: Option<IpNet>) -> Result<String, IpnetError> {
    use nix::ifaddrs::getifaddrs;
    let addrs = getifaddrs().map_err(|_| IpnetError::NoLocalIp)?;
    for addr in addrs {
        let Some(sockaddr) = addr.address else {
            continue;
        };
        let Some(sin) = sockaddr.as_sockaddr_in() else {
            continue;
        };
        let ip = sin.ip();
        // 127.0.0.0/8 整段都是回环，不能只挡 127.0.0.1
        if ip.is_loopback() || ip.is_unspecified() {
            continue;
        }
        if let Some(ref net) = sub_net
            && !net.contains(&std::net::IpAddr::V4(ip))
        {
            continue;
        }
        return Ok(ip.to_string());
    }
    Err(IpnetError::NoLocalIp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn net(s: &str) -> IpNet {
        IpNet::from_str(s).unwrap()
    }

    #[test]
    fn exact_networks_sort_first() {
        let host = net("192.168.1.10/32");
        let subnet = net("192.168.1.0/24");
        assert!(is_exact(&host));
        assert!(!is_exact(&subnet));
        assert_eq!(com_ip(&host, &subnet), Ordering::Less);
        assert_eq!(com_ip(&subnet, &host), Ordering::Greater);
        // 同为精确 / 同为非精确时按 IpNet 自身顺序
        assert_eq!(
            com_ip(&net("10.0.0.1/32"), &net("10.0.0.2/32")),
            net("10.0.0.1/32").cmp(&net("10.0.0.2/32"))
        );
    }

    #[test]
    fn ipv6_exact_prefix_is_128() {
        assert!(is_exact(&net("::1/128")));
        assert!(!is_exact(&net("::/0")));
    }
}
