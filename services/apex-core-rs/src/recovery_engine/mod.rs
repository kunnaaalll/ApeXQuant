use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartPolicy {
    Never,
    Immediate,
    LinearBackoff,
    ExponentialBackoff,
}

#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    pub max_restarts: u32,
    pub base_backoff_ms: u64,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_restarts: 5,
            base_backoff_ms: 1000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServiceRecoveryState {
    pub restart_count: u32,
    pub last_restart_timestamp_ms: u64,
}

#[derive(Default, Debug)]
pub struct RecoveryEngine {
    policies: HashMap<String, RestartPolicy>,
    configs: HashMap<String, RecoveryConfig>,
    recovery_states: HashMap<String, ServiceRecoveryState>,
}

impl RecoveryEngine {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            configs: HashMap::new(),
            recovery_states: HashMap::new(),
        }
    }

    pub fn configure_policy(&mut self, service_id: &str, policy: RestartPolicy, config: RecoveryConfig) {
        self.policies.insert(service_id.to_string(), policy);
        self.configs.insert(service_id.to_string(), config);
    }

    pub fn calculate_backoff(&mut self, service_id: &str, current_timestamp_ms: u64) -> Result<u64, &'static str> {
        let policy = self.policies.get(service_id).unwrap_or(&RestartPolicy::Never);
        let config = self.configs.get(service_id).cloned().unwrap_or_default();

        let state = self.recovery_states.entry(service_id.to_string()).or_insert(ServiceRecoveryState {
            restart_count: 0,
            last_restart_timestamp_ms: 0,
        });

        if state.restart_count >= config.max_restarts {
            return Err("Restart budget exceeded");
        }

        let backoff = match policy {
            RestartPolicy::Never => return Err("Restart policy is Never"),
            RestartPolicy::Immediate => 0,
            RestartPolicy::LinearBackoff => config.base_backoff_ms * (state.restart_count as u64 + 1),
            RestartPolicy::ExponentialBackoff => config.base_backoff_ms * (2_u64.pow(state.restart_count)),
        };

        state.restart_count += 1;
        state.last_restart_timestamp_ms = current_timestamp_ms;

        Ok(backoff)
    }

    pub fn get_restart_count(&self, service_id: &str) -> u32 {
        self.recovery_states.get(service_id).map(|s| s.restart_count).unwrap_or(0)
    }

    pub fn reset_recovery_state(&mut self, service_id: &str) {
        if let Some(state) = self.recovery_states.get_mut(service_id) {
            state.restart_count = 0;
            state.last_restart_timestamp_ms = 0;
        }
    }
}
