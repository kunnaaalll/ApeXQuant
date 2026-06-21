use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrokerRole {
    Primary,
    Secondary,
    Standby,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    pub broker_id: String,
    pub role: BrokerRole,
    pub priority: u32,
}
