use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceMetrics {
    pub startup_time_ms: u64,
    pub restart_count: u32,
    pub event_lag_ms: u64,
    pub throughput_tps: u32,
    pub memory_pressure_pct: u8, // 0-100
    pub queue_depth: u32,
}

#[derive(Default, Debug)]
pub struct ObservabilityEngine {
    metrics: HashMap<String, ServiceMetrics>,
}

impl ObservabilityEngine {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    pub fn record_startup_time(&mut self, service_id: &str, time_ms: u64) {
        let entry = self.metrics.entry(service_id.to_string()).or_default();
        entry.startup_time_ms = time_ms;
    }

    pub fn record_restart(&mut self, service_id: &str) {
        let entry = self.metrics.entry(service_id.to_string()).or_default();
        entry.restart_count += 1;
    }

    pub fn record_event_lag(&mut self, service_id: &str, lag_ms: u64) {
        let entry = self.metrics.entry(service_id.to_string()).or_default();
        entry.event_lag_ms = lag_ms;
    }

    pub fn record_throughput(&mut self, service_id: &str, tps: u32) {
        let entry = self.metrics.entry(service_id.to_string()).or_default();
        entry.throughput_tps = tps;
    }

    pub fn record_memory_pressure(&mut self, service_id: &str, pct: u8) {
        let entry = self.metrics.entry(service_id.to_string()).or_default();
        entry.memory_pressure_pct = pct.min(100);
    }

    pub fn record_queue_depth(&mut self, service_id: &str, depth: u32) {
        let entry = self.metrics.entry(service_id.to_string()).or_default();
        entry.queue_depth = depth;
    }

    pub fn get_metrics(&self, service_id: &str) -> Option<&ServiceMetrics> {
        self.metrics.get(service_id)
    }
}
