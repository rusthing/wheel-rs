//! # 序列化和反序列化 IpNet 类型
//!
//! 此模块提供了对 IpNet 类型的自定义序列化和反序列化实现。
//! 序列化时将 IpNet 转换为字符串格式（如 "192.168.1.0/24"），
//! 反序列化时将字符串解析为 IpNet。
//!
//! ## 示例
//!
//! ```
//! use ipnet::IpNet;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Config {
//!     #[serde(with = "wheel_rs::serde::ipnet_serde")]
//!     sub_net: IpNet,
//! }
//! ```

use ipnet::IpNet;
use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

/// # IpNet 序列化
///
/// 将 IpNet 序列化为字符串格式（如 "192.168.1.0/24"）。
pub fn serialize<S>(ipnet: &IpNet, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&ipnet.to_string())
}

/// # IpNet 反序列化
///
/// 将字符串格式的 CIDR 反序列化为 IpNet 类型。
/// 支持的格式如 "192.168.1.0/24"、"10.0.0.0/8"、"::1/128" 等。
pub fn deserialize<'de, D>(deserializer: D) -> Result<IpNet, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    IpNet::from_str(&s).map_err(serde::de::Error::custom)
}
