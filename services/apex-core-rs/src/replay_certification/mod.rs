use crate::event_sourcing::EventStore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayCertificationResult {
    pub is_identical: bool,
    pub original_hash: String,
    pub replay_hash: String,
}

#[derive(Default, Debug)]
pub struct ReplayCertifier {}

impl ReplayCertifier {
    pub fn new() -> Self {
        Self {}
    }

    pub fn certify_replay(
        &self,
        original_store: &EventStore,
        replay_store: &EventStore,
    ) -> Result<ReplayCertificationResult, &'static str> {
        let original_snapshot = original_store.take_system_snapshot()?;
        let replay_snapshot = replay_store.take_system_snapshot()?;

        let is_identical = original_snapshot.state_hash == replay_snapshot.state_hash
            && original_snapshot.sequence_number == replay_snapshot.sequence_number;

        Ok(ReplayCertificationResult {
            is_identical,
            original_hash: original_snapshot.state_hash,
            replay_hash: replay_snapshot.state_hash,
        })
    }
}
