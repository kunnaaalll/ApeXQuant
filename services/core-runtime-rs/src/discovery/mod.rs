use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryRecord {
    pub service_id: String,
    pub endpoints: Vec<String>,
    pub capabilities: Vec<String>,
}

#[derive(Default, Debug)]
pub struct DiscoveryService {
    records: HashMap<String, DiscoveryRecord>,
}

impl DiscoveryService {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    pub fn register(&mut self, record: DiscoveryRecord) -> Result<(), &'static str> {
        let _ = self.records.insert(record.service_id.clone(), record);
        Ok(())
    }

    pub fn discover_by_capability(&self, capability: &str) -> Vec<DiscoveryRecord> {
        self.records
            .values()
            .filter(|r| r.capabilities.iter().any(|c| c == capability))
            .cloned()
            .collect()
    }
}
