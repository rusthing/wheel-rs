use serde::de::Unexpected;
use serde::{Deserialize, Deserializer, Serializer};
use tracing_appender::rolling::Rotation;

pub fn serialize<S>(value: &Rotation, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let rotation_str = match value {
        &Rotation::WEEKLY => "weekly",
        &Rotation::DAILY => "daily",
        &Rotation::HOURLY => "hourly",
        &Rotation::MINUTELY => "minutely",
        &Rotation::NEVER => "never",
    };
    serializer.serialize_str(rotation_str)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Rotation, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    let log_rotation = s.trim().to_lowercase();
    Ok(match log_rotation.as_str() {
        "weekly" => Rotation::WEEKLY,
        "daily" => Rotation::DAILY,
        "hourly" => Rotation::HOURLY,
        "minutely" => Rotation::MINUTELY,
        "never" => Rotation::NEVER,
        _ => Err(serde::de::Error::invalid_value(
            Unexpected::Str(&log_rotation),
            &"weekly/daily/hourly/minutely/never",
        ))?,
    })
}
