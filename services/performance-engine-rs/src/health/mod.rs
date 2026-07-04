use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub service_name: String,
}

impl HealthStatus {
    pub fn check() -> Self {
        Self {
            healthy: true,
            service_name: "performance-engine".to_string(),
        }
    }
}
