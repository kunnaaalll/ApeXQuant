use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use dashmap::DashMap;
use time::OffsetDateTime;

use super::errors::PortfolioError;
use super::events::PortfolioEvent;
use super::snapshot::{PortfolioSnapshot, SnapshotFrequency};
use super::state::PortfolioState;

#[derive(Debug, Clone)]
pub struct PortfolioRegistry {
    state: Arc<RwLock<PortfolioState>>,
    version: Arc<AtomicU64>,
    snapshots: Arc<DashMap<SnapshotFrequency, Vec<PortfolioSnapshot>>>,
}

impl Default for PortfolioRegistry {
    fn default() -> Self {
        let snapshots = DashMap::new();
        snapshots.insert(SnapshotFrequency::Realtime, Vec::new());
        snapshots.insert(SnapshotFrequency::M1, Vec::new());
        snapshots.insert(SnapshotFrequency::M5, Vec::new());
        snapshots.insert(SnapshotFrequency::M15, Vec::new());
        snapshots.insert(SnapshotFrequency::H1, Vec::new());
        snapshots.insert(SnapshotFrequency::D1, Vec::new());

        Self {
            state: Arc::new(RwLock::new(PortfolioState::new())),
            version: Arc::new(AtomicU64::new(0)),
            snapshots: Arc::new(snapshots),
        }
    }
}

impl PortfolioRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Read-only access to the current state.
    pub fn get_state(&self) -> Result<PortfolioState, PortfolioError> {
        self.state.read().map(|guard| guard.clone()).map_err(|_| PortfolioError::SystemError("Lock poisoned".to_string()))
    }

    /// Get the current version of the portfolio state.
    pub fn get_version(&self) -> u64 {
        self.version.load(Ordering::SeqCst)
    }

    /// Retrieve historical snapshots for a specific frequency.
    pub fn get_snapshots(&self, freq: SnapshotFrequency) -> Vec<PortfolioSnapshot> {
        if let Some(entry) = self.snapshots.get(&freq) {
            entry.clone()
        } else {
            Vec::new()
        }
    }

    /// Apply a portfolio event.
    /// This will mutate the state, ensure invariants hold, increment the version,
    /// and generate a real-time snapshot.
    pub fn dispatch(&self, event: PortfolioEvent) -> Result<PortfolioSnapshot, PortfolioError> {
        let timestamp = OffsetDateTime::now_utc();
        
        let mut state_guard = self.state.write().map_err(|_| PortfolioError::SystemError("Lock poisoned".to_string()))?;
        
        // Try applying event
        state_guard.apply_event(&event, timestamp)?;

        // If we reach here, the transition was valid and invariants hold.
        let new_version = self.version.fetch_add(1, Ordering::SeqCst) + 1;
        let new_state = state_guard.clone();

        // Drop the write lock as early as possible before we do snapshotting work.
        drop(state_guard);

        let snapshot = PortfolioSnapshot::new(new_version, new_state, event, timestamp);

        // Store real-time snapshot
        if let Some(mut realtime_snapshots) = self.snapshots.get_mut(&SnapshotFrequency::Realtime) {
            realtime_snapshots.push(snapshot.clone());
        }

        // Logic for rolling up snapshots into M1, M5, etc. can be triggered here or 
        // by an external cron worker that reads Realtime snapshots.
        // For Phase 1, the structures fully support this.

        Ok(snapshot)
    }

    /// Explicitly store a roll-up snapshot for a specific frequency.
    pub fn store_periodic_snapshot(&self, freq: SnapshotFrequency, snapshot: PortfolioSnapshot) {
        if let Some(mut freq_snapshots) = self.snapshots.get_mut(&freq) {
            freq_snapshots.push(snapshot);
        }
    }
}
