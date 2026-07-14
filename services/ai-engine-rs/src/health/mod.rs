use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub database: String,
    pub redis: String,
    pub model_registry: String,
    pub feature_store: String,
    pub embedding_store: String,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            status: "OK".to_string(),
            database: "OK".to_string(),
            redis: "OK".to_string(),
            model_registry: "OK".to_string(),
            feature_store: "OK".to_string(),
            embedding_store: "OK".to_string(),
        }
    }
}
