use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEngineConfig {
    pub server_addr: String,
    pub database_url: String,
    pub redis_url: String,
    pub event_bus_url: String,
    pub metrics_port: u16,
}

impl Default for AiEngineConfig {
    fn default() -> Self {
        Self {
            server_addr: "0.0.0.0:50051".to_string(),
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            redis_url: "redis://localhost:6379".to_string(),
            event_bus_url: "kafka://localhost:9092".to_string(),
            metrics_port: 9090,
        }
    }
}

impl AiEngineConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder()
            .add_source(config::Environment::with_prefix("AI_ENGINE").separator("__"));

        // In a real scenario we might add config file support:
        // .add_source(config::File::with_name("config/ai_engine").required(false))

        builder.build()?.try_deserialize()
    }
}
