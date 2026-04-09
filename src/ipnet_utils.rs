use ipnet::IpNet;
use std::cmp::Ordering;

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
