//! Storage Module
//!
//! Persistence for simulation events, snapshots, reports, and parameter sets.

use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub session_id: String,
    pub timestamp: OffsetDateTime,
    pub state_snapshot: Vec<u8>,
}

pub trait StorageEngine {
    fn save_checkpoint(&mut self, checkpoint: Checkpoint) -> Result<(), &'static str>;
    fn load_checkpoint(&self, session_id: &str, timestamp: OffsetDateTime) -> Result<Option<Checkpoint>, &'static str>;
    fn save_offset(&mut self, session_id: &str, offset: u64) -> Result<(), &'static str>;
    fn get_offset(&self, session_id: &str) -> Result<u64, &'static str>;
}
