// /Users/zbz/workspace/rusthing/wheel-rs/src/serde/vec_regex_serde.rs
use regex::Regex;
use serde::{
    de::{self, Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
};
use std::fmt;

pub fn serialize<S>(vec: &Vec<Regex>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(vec.len()))?;
    for element in vec {
        seq.serialize_element(element.as_str())?;
    }
    seq.end()
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Regex>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(RegexVecVisitor)
}

struct RegexVecVisitor;

impl<'de> Visitor<'de> for RegexVecVisitor {
    type Value = Vec<Regex>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a regex string or array of regex strings")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let regex =
            Regex::new(value).map_err(|e| de::Error::custom(format!("invalid regex: {e}")))?;
        Ok(vec![regex])
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
            let regex = Regex::new(&element)
                .map_err(|e| de::Error::custom(format!("invalid regex '{element}': {e}")))?;
            vec.push(regex);
        }
        Ok(vec)
    }
}
