use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::net::ToSocketAddrs;
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
        let (host, port) = if let Some(inner) = s.strip_prefix('[') {
            let bracket_end = inner
                .find(']')
                .ok_or_else(|| AddrError::Parse(format!("Invalid address: {s}")))?;
            let host = &inner[..bracket_end];
            let rest = &inner[bracket_end + 1..];
            let port = if let Some(port_str) = rest.strip_prefix(':') {
                Some(
                    port_str
                        .parse::<u16>()
                        .map_err(|_| AddrError::Parse(format!("Invalid port: {s}")))?,
                )
            } else if rest.is_empty() {
                None
            } else {
                return Err(AddrError::Parse(format!("Invalid address: {s}")));
            };
            (host.to_string(), port)
        } else if s.matches(':').count() > 1 {
            (s.to_string(), None)
        } else if let Some((host, port_str)) = s.split_once(':') {
            let port = port_str
                .parse::<u16>()
                .map_err(|_| AddrError::Parse(format!("Invalid port: {s}")))?;
            (host.to_string(), Some(port))
        } else {
            (s.to_string(), None)
        };

        if host.is_empty() {
            return Err(AddrError::Parse(format!("Invalid address: {s}")));
        }

        Ok(Addr { host, port })
    }
}

impl Display for Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(port) = self.port {
            if self.host.contains(':') {
                write!(f, "[{}]:{}", self.host, port)
            } else {
                write!(f, "{}:{}", self.host, port)
            }
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

impl ToSocketAddrs for Addr {
    type Iter = std::vec::IntoIter<std::net::SocketAddr>;

    fn to_socket_addrs(&self) -> Result<Self::Iter, std::io::Error> {
        let port = self.port.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("missing port for host: {}", self.host),
            )
        })?;
        (self.host.as_str(), port).to_socket_addrs()
    }
}
