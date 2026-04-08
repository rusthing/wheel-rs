use crate::urn_utils::Urn;
use serde::{
    de::{self, Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
};
use std::fmt;

pub fn serialize<S>(vec: &Vec<Urn>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(vec.len()))?;
    for element in vec {
        let urn_str = format!("{}:{}", element.method.to_string(), element.url);
        seq.serialize_element(&urn_str)?;
    }
    seq.end()
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Urn>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(UrnVecVisitor)
}

struct UrnVecVisitor;

impl<'de> Visitor<'de> for UrnVecVisitor {
    type Value = Vec<Urn>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string, comma-separated string, or array of strings")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(value.to_string())
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| Urn::new(s))
            .collect())
    }

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
            vec.push(Urn::new(element));
        }
        Ok(vec)
    }
}
