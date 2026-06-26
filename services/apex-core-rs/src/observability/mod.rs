use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_services: u32,
    pub active_events_sec: u64,
    pub average_queue_depth: u64,
}

pub struct ObservabilityManager {
    // internals omitted
}

impl Default for ObservabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ObservabilityManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn record_metric(&self, _name: &str, _value: u64) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn get_system_metrics(&self) -> SystemMetrics {
        SystemMetrics {
            total_services: 0,
            active_events_sec: 0,
            average_queue_depth: 0,
        }
    }
}
