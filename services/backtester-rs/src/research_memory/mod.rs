use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    FailedIdea,
    RetiredStrategy,
    DiscoveredRegime,
    MarketAnomaly,
}

#[derive(Debug, Clone)]
pub struct ResearchMemoryEntry {
    pub id: Uuid,
    pub memory_type: MemoryType,
    pub reference_id: Uuid, // e.g., strategy_id or hypothesis_id
    pub recorded_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

pub trait ResearchMemoryStore {
    fn record(&mut self, entry: ResearchMemoryEntry);
    fn query(&self, memory_type: MemoryType) -> Vec<&ResearchMemoryEntry>;
    fn check_duplicate_idea(&self, hypothesis_hash: &str) -> bool;
}

pub struct InMemoryResearchStore {
    entries: Vec<ResearchMemoryEntry>,
    hashes: std::collections::HashSet<String>,
}

impl InMemoryResearchStore {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            hashes: std::collections::HashSet::new(),
        }
    }
}

impl Default for InMemoryResearchStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ResearchMemoryStore for InMemoryResearchStore {
    fn record(&mut self, entry: ResearchMemoryEntry) {
        self.entries.push(entry);
    }

    fn query(&self, memory_type: MemoryType) -> Vec<&ResearchMemoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.memory_type == memory_type)
            .collect()
    }

    fn check_duplicate_idea(&self, hypothesis_hash: &str) -> bool {
        self.hashes.contains(hypothesis_hash)
    }
}
