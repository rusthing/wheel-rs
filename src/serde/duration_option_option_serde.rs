// duration_option_option_serde.rs
use serde::{Deserialize, Deserializer, Serializer};
use std::time::Duration;

pub fn serialize<S>(value: &Option<Option<Duration>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(Some(d)) => serializer.serialize_str(&format!("{}s", d.as_secs())),
        Some(None) | None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(d: D) -> Result<Option<Option<Duration>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<Option<String>> = Option::<Option<String>>::deserialize(d)?;
    match opt {
        None => Ok(None),
        Some(None) => Ok(Some(None)),
        Some(Some(s)) => {
            let dur = humantime::parse_duration(&s).map_err(serde::de::Error::custom)?;
            Ok(Some(Some(dur)))
        }
    }
}
