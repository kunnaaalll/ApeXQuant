use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub shadow_mode_enabled: bool,
    pub deterministic_replay_enabled: bool,
    pub strict_ordering_enforced: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            shadow_mode_enabled: false,
            deterministic_replay_enabled: true,
            strict_ordering_enforced: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfiguration {
    pub env_name: String,
    pub base_url: String,
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfiguration {
    pub engine_id: String,
    pub version: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfiguration {
    pub environment: EnvironmentConfiguration,
    pub engines: Vec<EngineConfiguration>,
}

impl SystemConfiguration {
    pub fn new(env_name: String, base_url: String) -> Self {
        Self {
            environment: EnvironmentConfiguration {
                env_name,
                base_url,
                features: FeatureFlags::default(),
            },
            engines: Vec::new(),
        }
    }
}
