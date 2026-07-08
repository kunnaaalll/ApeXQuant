use crate::brokers::broker::{OrderState, PositionState};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub positions: Vec<PositionState>,
    pub orders: Vec<OrderState>,
    pub timestamp: u64,
}

#[derive(Default)]
pub struct PersistenceEngine {
    latest_snapshot: Option<StateSnapshot>,
}

impl PersistenceEngine {
    pub fn new() -> Self {
        Self {
            latest_snapshot: None,
        }
    }

    pub fn persist(&mut self, positions: Vec<PositionState>, orders: Vec<OrderState>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.latest_snapshot = Some(StateSnapshot {
            positions,
            orders,
            timestamp,
        });
    }

    pub fn load_latest(&self) -> Option<StateSnapshot> {
        self.latest_snapshot.clone()
    }
}
