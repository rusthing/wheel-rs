// ... existing code ...
use ipnet::IpNet;
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use std::fmt;

pub fn serialize<S>(vec: &Vec<IpNet>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let string = vec
        .iter()
        .map(|ipnet| ipnet.to_string())
        .collect::<Vec<_>>()
        .join(",");
    serializer.serialize_str(&string)
}

/// # 将 JSON 数据反序列化为 `Vec<IpNet>` 类型
///
/// ## 支持的格式
/// - `"192.168.1.0/24"` -> `vec![IpNet::from_str("192.168.1.0/24").unwrap()]`
/// - `["192.168.1.0/24", "10.0.0.0/8"]` -> `vec![IpNet::from_str("192.168.1.0/24").unwrap(), IpNet::from_str("10.0.0.0/8").unwrap()]`
/// - `null` -> `vec![]`
///
/// ## 示例
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use serde_json;
///
/// #[derive(Serialize, Deserialize)]
/// struct Example {
/// #[serde(deserialize_with = "crate::serde::vec_serde::deserialize_ipnet_vec")]
/// networks: Vec<IpNet>,
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
            .map(|s| s.trim().to_string())
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
