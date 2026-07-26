use hickory_resolver::config::ResolveHosts;
use serde::de::Unexpected;
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(resolve_hosts: &Option<ResolveHosts>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match resolve_hosts {
        Some(rh) => {
            let s = match rh {
                ResolveHosts::Auto => "Auto",
                ResolveHosts::Always => "Always",
                ResolveHosts::Never => "Never",
            };
            serializer.serialize_str(s)
        }
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ResolveHosts>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => Ok(Some(match s.to_lowercase().as_str() {
            "auto" => ResolveHosts::Auto,
            "always" => ResolveHosts::Always,
            "never" => ResolveHosts::Never,
            _ => Err(serde::de::Error::invalid_value(
                Unexpected::Str(&s),
                &"auto, always, never",
            ))?,
        })),
        None => Ok(None),
    }
}
