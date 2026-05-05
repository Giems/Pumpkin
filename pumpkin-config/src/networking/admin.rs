use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};

#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct AdminPanelConfig {
    pub enabled: bool,
    pub address: SocketAddr,
    /// Bearer token required for all API requests (Authorization: Bearer <token>).
    /// Leave empty to disable auth (not recommended for production).
    pub token: String,
}

impl Default for AdminPanelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            address: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 9000),
            token: String::new(),
        }
    }
}
