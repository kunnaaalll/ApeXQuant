use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use time::OffsetDateTime;

use super::errors::ExposureError;
use super::events::ExposureEvent;
use super::snapshot::{ExposureSnapshot, ExposureSnapshotFrequency};
use super::state::ExposureState;

#[derive(Debug, Clone)]
pub struct ExposureRegistry {
    state: Arc<RwLock<ExposureState>>,
    version: Arc<AtomicU64>,
    snapshots: Arc<DashMap<ExposureSnapshotFrequency, Vec<ExposureSnapshot>>>,
}

impl Default for ExposureRegistry {
    fn default() -> Self {
        let snapshots = DashMap::new();
        snapshots.insert(ExposureSnapshotFrequency::Realtime, Vec::new());
        snapshots.insert(ExposureSnapshotFrequency::M1, Vec::new());
        snapshots.insert(ExposureSnapshotFrequency::M5, Vec::new());
        snapshots.insert(ExposureSnapshotFrequency::M15, Vec::new());
        snapshots.insert(ExposureSnapshotFrequency::H1, Vec::new());
        snapshots.insert(ExposureSnapshotFrequency::D1, Vec::new());

        Self {
            state: Arc::new(RwLock::new(ExposureState::new())),
            version: Arc::new(AtomicU64::new(0)),
            snapshots: Arc::new(snapshots),
        }
    }
}

impl ExposureRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_state(&self) -> Result<ExposureState, ExposureError> {
        self.state
            .read()
            .map(|guard| guard.clone())
            .map_err(|_| ExposureError::SystemError("Lock poisoned".to_string()))
    }

    pub fn get_version(&self) -> u64 {
        self.version.load(Ordering::SeqCst)
    }

    pub fn get_snapshots(&self, freq: ExposureSnapshotFrequency) -> Vec<ExposureSnapshot> {
        if let Some(entry) = self.snapshots.get(&freq) {
            entry.clone()
        } else {
            Vec::new()
        }
    }

    pub fn dispatch(&self, event: ExposureEvent) -> Result<ExposureSnapshot, ExposureError> {
        let timestamp = OffsetDateTime::now_utc();
        let mut state_guard = self
            .state
            .write()
            .map_err(|_| ExposureError::SystemError("Lock poisoned".to_string()))?;

        state_guard.apply_event(&event, timestamp)?;

        let new_version = self.version.fetch_add(1, Ordering::SeqCst) + 1;
        let new_state = state_guard.clone();

        drop(state_guard);

        let snapshot = ExposureSnapshot::new(new_version, new_state, event, timestamp);

        if let Some(mut realtime_snapshots) =
            self.snapshots.get_mut(&ExposureSnapshotFrequency::Realtime)
        {
            realtime_snapshots.push(snapshot.clone());
        }

        Ok(snapshot)
    }
}
