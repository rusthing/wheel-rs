//! # 序列化和反序列化 Option<IpNet> 类型
//!
//! 此模块提供了对 Option<IpNet> 类型的自定义序列化和反序列化实现。
//! 序列化时 Some(IpNet) 会被转换为字符串格式（如 "192.168.1.0/24"），
//! None 值会被序列化为 null。
//!
//! ## 示例
//!
//! ```
//! use ipnet::IpNet;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Config {
//!     #[serde(with = "wheel_rs::serde::ipnet_option_serde")]
//!     sub_net: Option<IpNet>,
//! }
//! ```

use ipnet::IpNet;
use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

/// # Option<IpNet> 序列化
///
/// 将 Option<IpNet> 序列化为字符串格式或 null。
/// - Some(ipnet) 序列化为字符串 "192.168.1.0/24"
/// - None 序列化为 null
pub fn serialize<S>(ipnet: &Option<IpNet>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match ipnet {
        Some(ip) => serializer.serialize_str(&ip.to_string()),
        None => serializer.serialize_none(),
    }
}

/// # Option<IpNet> 反序列化
///
/// 将字符串或 null 反序列化为 Option<IpNet> 类型。
/// - 字符串 "192.168.1.0/24" 解析为 Some(IpNet)
/// - null 解析为 None
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<IpNet>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_option(IpNetOptionVisitor)
}

struct IpNetOptionVisitor;

impl<'de> serde::de::Visitor<'de> for IpNetOptionVisitor {
    type Value = Option<IpNet>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a CIDR string like \"192.168.1.0/24\" or null")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        IpNet::from_str(&s)
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
}
