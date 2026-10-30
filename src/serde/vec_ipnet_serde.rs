//! # 自定义序列化/反序列化器，用于处理 `Vec<IpNet>` 类型的数据
//!
//! 此模块提供 `Vec<IpNet>` 类型的自定义序列化和反序列化实现。
//! 序列化输出为字符串数组（如 `["10.0.0.0/8", "192.168.0.0/16"]`），与
//! [`crate::serde::vec_serde`]、[`crate::serde::vec_addr_serde`] 等保持一致；
//! 反序列化时额外兼容单个网段字符串与逗号分隔字符串，便于手写配置。

use ipnet::IpNet;
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use std::fmt;

/// # 将 `Vec<IpNet>` 序列化为字符串数组
///
/// 每个网段以 CIDR 字符串形式输出，空向量输出为 `[]`。
///
/// ## 参数
/// - `vec`: 待序列化的网段向量。
/// - `serializer`: 序列化器。
///
/// ## 返回值
/// 返回序列化结果，序列化失败时返回 `S::Error`。
///
/// ## 示例
///
/// ```
/// use ipnet::IpNet;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Example {
///     #[serde(serialize_with = "wheel_rs::serde::vec_ipnet_serde::serialize")]
///     networks: Vec<IpNet>,
/// }
///
/// let json = serde_json::to_string(&Example {
///     networks: vec!["10.0.0.0/8".parse().unwrap()],
/// })
/// .unwrap();
/// assert_eq!(json, r#"{"networks":["10.0.0.0/8"]}"#);
/// ```
pub fn serialize<S>(vec: &[IpNet], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(vec.len()))?;
    for ipnet in vec {
        seq.serialize_element(&ipnet.to_string())?;
    }
    seq.end()
}

/// # 将 JSON 数据反序列化为 `Vec<IpNet>` 类型
///
/// ## 支持的格式
/// - `"192.168.1.0/24"` -> `vec![192.168.1.0/24]`（单个网段）
/// - `"192.168.1.0/24,10.0.0.0/8"` -> 两个元素（逗号分隔，各项会 trim，空项被丢弃）
/// - `["192.168.1.0/24", "10.0.0.0/8"]` -> 两个元素（数组）
///
/// ## 示例
/// ```rust
/// use ipnet::IpNet;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Example {
///     #[serde(deserialize_with = "wheel_rs::serde::vec_ipnet_serde::deserialize")]
///     networks: Vec<IpNet>,
/// }
/// ```
pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<IpNet>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(IpNetVecVisitor)
}

/// # 用于处理 IPNet 字符串或 IPNet 数组的访问器
struct IpNetVecVisitor;

impl<'de> Visitor<'de> for IpNetVecVisitor {
    type Value = Vec<IpNet>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string, comma-separated string, or array of IPNet strings")
    }

    /// # 处理字符串引用（&str）类型
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(value.to_string())
    }

    /// # 处理字符串（String）类型
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<IpNet>().map_err(de::Error::custom))
            .collect()
    }

    /// # 处理序列（数组）类型
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        while let Some(element) = seq.next_element::<String>()? {
            let element = element.trim().to_string();
            if element.is_empty() {
                continue;
            }
            let ipnet = element.parse::<IpNet>().map_err(de::Error::custom)?;
            vec.push(ipnet);
        }
        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Holder {
        #[serde(with = "super")]
        nets: Vec<IpNet>,
    }

    fn nets(s: &[&str]) -> Vec<IpNet> {
        s.iter().map(|s| s.parse().unwrap()).collect()
    }

    #[test]
    fn serializes_as_array() {
        let h = Holder {
            nets: nets(&["10.0.0.0/8", "192.168.0.0/16"]),
        };
        assert_eq!(
            serde_json::to_string(&h).unwrap(),
            r#"{"nets":["10.0.0.0/8","192.168.0.0/16"]}"#
        );
    }

    #[test]
    fn empty_serializes_as_empty_array() {
        let h = Holder { nets: vec![] };
        assert_eq!(serde_json::to_string(&h).unwrap(), r#"{"nets":[]}"#);
        // 空数组反序列化回来仍是空向量
        let back: Holder = serde_json::from_str(r#"{"nets":[]}"#).unwrap();
        assert!(back.nets.is_empty());
    }

    #[test]
    fn deserializes_all_supported_shapes() {
        let array: Holder =
            serde_json::from_str(r#"{"nets":["10.0.0.0/8","192.168.0.0/16"]}"#).unwrap();
        assert_eq!(array.nets, nets(&["10.0.0.0/8", "192.168.0.0/16"]));

        let comma: Holder =
            serde_json::from_str(r#"{"nets":"10.0.0.0/8, 192.168.0.0/16"}"#).unwrap();
        assert_eq!(comma.nets, nets(&["10.0.0.0/8", "192.168.0.0/16"]));

        let single: Holder = serde_json::from_str(r#"{"nets":"10.0.0.0/8"}"#).unwrap();
        assert_eq!(single.nets, nets(&["10.0.0.0/8"]));
    }

    #[test]
    fn round_trip_preserves_values() {
        let h = Holder {
            nets: nets(&["::1/128", "172.16.0.0/12"]),
        };
        let json = serde_json::to_string(&h).unwrap();
        let back: Holder = serde_json::from_str(&json).unwrap();
        assert_eq!(back.nets, h.nets);
    }

    #[test]
    fn invalid_element_is_reported() {
        assert!(serde_json::from_str::<Holder>(r#"{"nets":["not-a-cidr"]}"#).is_err());
        assert!(serde_json::from_str::<Holder>(r#"{"nets":"not-a-cidr"}"#).is_err());
    }
}
