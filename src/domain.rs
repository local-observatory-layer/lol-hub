use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_host: String,
    pub bind_port: u16,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, std::num::ParseIntError> {
        let bind_host =
            std::env::var("LOL_HUB_BIND_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let bind_port = match std::env::var("LOL_HUB_BIND_PORT") {
            Ok(v) => v.parse::<u16>()?,
            Err(_) => 3060,
        };

        Ok(Self {
            bind_host,
            bind_port,
        })
    }

    pub fn bind_addr(&self) -> Result<SocketAddr, std::io::Error> {
        let addr = format!("{}:{}", self.bind_host, self.bind_port);
        addr.to_socket_addrs()?.next().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "no address found")
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSample {
    pub device_id: Uuid,
    pub kind: String,
    pub data: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDescriptor {
    pub device_id: Uuid,
    pub name: String,
    pub layouts: HashMap<String, LayoutDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutDescriptor {
    pub width: u32,
    pub height: u32,
    pub color_mode: String,
    pub rotation: Option<u16>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    Sample(DeviceSample),
    Descriptor(DeviceDescriptor),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_defaults() {
        let cfg = AppConfig::from_env().unwrap();

        assert_eq!(cfg.bind_host, "127.0.0.1");
        assert_eq!(cfg.bind_port, 3060);
    }

    #[test]
    fn bind_addr_builds_socket_addr() {
        let cfg = AppConfig {
            bind_host: "127.0.0.1".into(),
            bind_port: 3060,
        };

        let addr = cfg.bind_addr().unwrap();

        assert_eq!(addr.to_string(), "127.0.0.1:3060");
    }

    #[test]
    fn invalid_host_returns_error() {
        let cfg = AppConfig {
            bind_host: "lol.invalid".into(),
            bind_port: 3060,
        };

        assert!(cfg.bind_addr().is_err());
    }
}
