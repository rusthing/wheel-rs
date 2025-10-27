use dns_lookup::lookup_host;
use log::info;
use std::net::IpAddr;

/// # 解析主机的字符串成IP地址
/// ## 参数
/// * `host` - 一个字符串切片，表示要解析的主机名或 IP 地址
/// ## 返回值
/// 如果解析成功，则返回一个 `IpAddr`; 如果解析失败，则返回一个包含错误信息的 `String`
/// ```
pub fn parse_host(host: &str) -> Result<IpAddr, String> {
    // 尝试直接解析为 IP 地址
    if let Ok(ip_addr) = host.parse::<IpAddr>() {
        return Ok(ip_addr);
    }

    // 不是 IP，尝试 DNS 解析
    let mut addrs = lookup_host(host).map_err(|e| format!("DNS lookup failed: {e}"))?;
    let ip_addr = addrs
        .next()
        .ok_or_else(|| format!("Failed to resolve hostname: {host}"))?;
    info!("Resolved hostname: {host} -> {}", ip_addr.to_string());
    Ok(ip_addr)
}

/// # 解析主机名和端口号字符串为 IP 地址和端口号
/// 支持格式:
/// - "192.168.1.1:8080"
/// - "example.com:8080"
/// - \["::1\]:8080" (IPv6)
/// - "192.168.1.1" (无端口号，默认端口为0)
/// - "example.com" (无端口号，默认端口为0)
/// ## 参数
/// * `host_port` - 包含主机名和可选端口号的字符串
/// ## 返回值
/// 如果解析成功，返回 (IpAddr, u16) 元组；如果解析失败，返回错误信息
/// 如果未提供端口号，则端口号默认为 0
pub fn parse_host_port(host_port: &str) -> Result<(IpAddr, u16), String> {
    // 处理 IPv6 地址带端口的情况 [::1]:8080
    if host_port.starts_with('[') {
        if let Some(pos) = host_port.find("]:") {
            let host = &host_port[1..pos];
            let port = host_port[pos + 2..].parse::<u16>()
                .map_err(|_| format!("Invalid port in address: {}", host_port))?;
            let ip_addr = parse_host(host)?;
            return Ok((ip_addr, port));
        }
    }
    
    // 处理一般的 host:port 格式
    if let Some(pos) = host_port.rfind(':') {
        let host = &host_port[..pos];
        let port = host_port[pos + 1..].parse::<u16>()
            .map_err(|_| format!("Invalid port in address: {}", host_port))?;
        let ip_addr = parse_host(host)?;
        Ok((ip_addr, port))
    } else {
        // 没有找到端口号，只解析主机名，默认端口为 0
        let ip_addr = parse_host(host_port)?;
        Ok((ip_addr, 0))
    }
}