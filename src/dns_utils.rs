use dns_lookup::lookup_host;
use log::info;
use std::net::IpAddr;

/// 解析主机的字符串成IP地址
/// # 参数
/// * `host` - 一个字符串切片，表示要解析的主机名或 IP 地址
/// # 返回值
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
