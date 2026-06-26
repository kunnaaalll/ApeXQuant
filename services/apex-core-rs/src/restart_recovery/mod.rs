use crate::event_sourcing::{EventStore, SystemEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryTestResult {
    pub success: bool,
    pub original_hash: String,
    pub recovered_hash: String,
}

#[derive(Default, Debug)]
pub struct RestartRecoveryTester {}

impl RestartRecoveryTester {
    pub fn new() -> Self {
        Self {}
    }

    pub fn force_engine_crash(&self) -> Result<(), &'static str> {
        // Mocking a forced engine crash
        // In a real environment, this might send a kill signal to an engine process
        Ok(())
    }

    pub fn rebuild_from_events(
        &self,
        original_events: &[SystemEvent],
    ) -> Result<EventStore, &'static str> {
        let mut new_store = EventStore::new();
        for event in original_events {
            new_store.append(event.clone())?;
        }
        Ok(new_store)
    }

    pub fn verify_recovered_state(
        &self,
        original_store: &EventStore,
        recovered_store: &EventStore,
    ) -> Result<RecoveryTestResult, &'static str> {
        let original_snapshot = original_store.take_system_snapshot()?;
        let recovered_snapshot = recovered_store.take_system_snapshot()?;

        let success = original_snapshot.state_hash == recovered_snapshot.state_hash
            && original_snapshot.sequence_number == recovered_snapshot.sequence_number;

        Ok(RecoveryTestResult {
            success,
            original_hash: original_snapshot.state_hash,
            recovered_hash: recovered_snapshot.state_hash,
        })
    }
}
