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
            status: "MODEL_UNAVAILABLE".to_string(),
            database: "NOT_CONNECTED".to_string(),
            redis: "NOT_CONNECTED".to_string(),
            model_registry: "MODEL_UNAVAILABLE".to_string(),
            feature_store: "NOT_CONNECTED".to_string(),
            embedding_store: "NOT_CONNECTED".to_string(),
        }
    }
}
