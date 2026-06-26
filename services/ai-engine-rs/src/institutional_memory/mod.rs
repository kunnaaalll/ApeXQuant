use std::collections::{HashSet, HashMap};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryCategory {
    FailedResearch,
    RetiredStrategy,
    DeadEdge,
    DiscoveredAnomaly,
    SuccessfulOptimization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub category: MemoryCategory,
    pub signature: String, // A hash or deterministic string describing the parameters/setup
    pub timestamp: u64,
    pub reasoning: String,
}

pub struct InstitutionalMemory {
    records: HashMap<MemoryCategory, Vec<MemoryRecord>>,
    signatures: HashSet<String>,
}

impl InstitutionalMemory {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            signatures: HashSet::new(),
        }
    }

    pub fn record_event(&mut self, record: MemoryRecord) {
        self.signatures.insert(record.signature.clone());
        self.records.entry(record.category.clone()).or_default().push(record);
    }

    pub fn check_duplicate_research(&self, research_signature: &str) -> bool {
        self.signatures.contains(research_signature)
    }

    pub fn retrieve_by_category(&self, category: MemoryCategory) -> Vec<MemoryRecord> {
        self.records.get(&category).cloned().unwrap_or_default()
    }
}

impl Default for InstitutionalMemory {
    fn default() -> Self {
        Self::new()
    }
}
