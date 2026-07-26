use hickory_resolver::config::ResolveHosts;
use serde::de::Unexpected;
use serde::{Deserialize, Deserializer};

pub fn serialize<S>(resolve_hosts: &ResolveHosts, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = match resolve_hosts {
        ResolveHosts::Auto => "Auto",
        ResolveHosts::Always => "Always",
        ResolveHosts::Never => "Never",
    };
    serializer.serialize_str(s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<ResolveHosts, D::Error>
where
    D: Deserializer<'de>,
{
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.to_lowercase().as_str() {
            "auto" => ResolveHosts::Auto,
            "always" => ResolveHosts::Always,
            "never" => ResolveHosts::Never,
            _ => Err(serde::de::Error::invalid_value(
                Unexpected::Str(&s),
                &"auto, always, never",
            ))?,
        })
    }
}
