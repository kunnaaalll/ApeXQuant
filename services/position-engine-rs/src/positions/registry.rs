use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::tracker::PositionTracker;

/// Thread-safe registry for actively managed positions in memory.
#[derive(Debug, Clone, Default)]
pub struct PositionRegistry {
    positions: Arc<DashMap<Uuid, PositionTracker>>,
}

impl PositionRegistry {
    pub fn new() -> Self {
        Self {
            positions: Arc::new(DashMap::new()),
        }
    }

    pub fn insert(&self, position: PositionTracker) {
        self.positions.insert(position.position_id, position);
    }

    pub fn get(&self, position_id: &Uuid) -> Option<PositionTracker> {
        self.positions.get(position_id).map(|p| p.clone())
    }

    pub fn remove(&self, position_id: &Uuid) -> Option<PositionTracker> {
        self.positions.remove(position_id).map(|(_, p)| p)
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}
