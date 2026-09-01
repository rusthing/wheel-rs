use ipnet::IpNet;
use std::cmp::Ordering;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IpnetError {
    #[error("no local ip address found")]
    NoLocalIp,
}

/// Compare two networks.
pub fn com_ip(a: &IpNet, b: &IpNet) -> Ordering {
    match (is_exact(a), is_exact(b)) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => a.cmp(b),
    }
}

/// Returns true if the network is exact.
pub fn is_exact(net: &IpNet) -> bool {
    match net {
        IpNet::V4(n) => n.prefix_len() == 32,
        IpNet::V6(n) => n.prefix_len() == 128,
    }
}

/// 获取本机首个非 loopback 的 IPv4 地址。
///
/// 通过遍历系统网卡接口获取，不依赖外网连通性。
pub fn get_local_ip() -> Result<String, IpnetError> {
    use nix::ifaddrs::getifaddrs;
    let addrs = getifaddrs().map_err(|_| IpnetError::NoLocalIp)?;
    for addr in addrs {
        if let Some(sockaddr) = addr.address {
            if let Some(sin) = sockaddr.as_sockaddr_in() {
                let ip = sin.ip().to_string();
                if ip != "127.0.0.1" {
                    return Ok(ip);
                }
            }
        }
    }
    Err(IpnetError::NoLocalIp)
}
