use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AddrError {
    #[error("Fail to parse Addr string: {0}")]
    Parse(String),
}

#[derive(Debug, Clone)]
pub struct Addr {
    pub host: String,
    pub port: Option<u16>,
}

impl Addr {
    pub fn new(host: String, port: Option<u16>) -> Addr {
        Addr { host, port }
    }

    pub fn from_str(s: &str) -> Result<Addr, AddrError> {
        let parts: Vec<_> = s.split(':').collect();
        let len = parts.len();
        let mut port = None;
        if len < 1 || len > 2 {
            return Err(AddrError::Parse(format!("Invalid address: {s}")));
        }
        let host = parts[0].to_string();
        if len == 2 {
            port = Some(
                parts[1]
                    .parse::<u16>()
                    .map_err(|_| AddrError::Parse(format!("Invalid port: {s}")))?,
            );
        }
        Ok(Addr { host, port })
    }
}

impl Display for Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(port) = self.port {
            write!(f, "{}:{}", self.host, port)
        } else {
            write!(f, "{}", self.host)
        }
    }
}

impl Serialize for Addr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Addr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Addr::from_str(&s).map_err(|e| serde::de::Error::custom(format!("{:?}", e)))
    }
}
