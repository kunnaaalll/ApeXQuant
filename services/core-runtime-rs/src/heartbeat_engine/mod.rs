use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeartbeatState {
    Healthy,
    Warning,
    Degraded,
    Critical,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    pub interval_ms: u64,
    pub warning_threshold_ms: u64,
    pub degraded_threshold_ms: u64,
    pub critical_threshold_ms: u64,
    pub offline_threshold_ms: u64,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_ms: 1000,
            warning_threshold_ms: 3000,
            degraded_threshold_ms: 5000,
            critical_threshold_ms: 10000,
            offline_threshold_ms: 30000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineHealthStatus {
    pub last_heartbeat_ms: u64,
    pub missed_heartbeats: u32,
    pub state: HeartbeatState,
}

#[derive(Default, Debug)]
pub struct HeartbeatEngine {
    config: HeartbeatConfig,
    health_statuses: HashMap<String, EngineHealthStatus>,
}

impl HeartbeatEngine {
    pub fn new(config: HeartbeatConfig) -> Self {
        Self {
            config,
            health_statuses: HashMap::new(),
        }
    }

    pub fn record_heartbeat(&mut self, service_id: &str, timestamp_ms: u64) {
        let status = self.health_statuses.entry(service_id.to_string()).or_insert(EngineHealthStatus {
            last_heartbeat_ms: timestamp_ms,
            missed_heartbeats: 0,
            state: HeartbeatState::Healthy,
        });
        
        status.last_heartbeat_ms = timestamp_ms;
        status.missed_heartbeats = 0;
        status.state = HeartbeatState::Healthy;
    }

    pub fn evaluate_health(&mut self, current_time_ms: u64) {
        for status in self.health_statuses.values_mut() {
            let elapsed = current_time_ms.saturating_sub(status.last_heartbeat_ms);
            
            if elapsed >= self.config.offline_threshold_ms {
                status.state = HeartbeatState::Offline;
            } else if elapsed >= self.config.critical_threshold_ms {
                status.state = HeartbeatState::Critical;
            } else if elapsed >= self.config.degraded_threshold_ms {
                status.state = HeartbeatState::Degraded;
            } else if elapsed >= self.config.warning_threshold_ms {
                status.state = HeartbeatState::Warning;
            } else {
                status.state = HeartbeatState::Healthy;
            }

            if elapsed >= self.config.interval_ms {
                status.missed_heartbeats = (elapsed / self.config.interval_ms) as u32;
            }
        }
    }

    pub fn get_health(&self, service_id: &str) -> Option<&EngineHealthStatus> {
        self.health_statuses.get(service_id)
    }
}
