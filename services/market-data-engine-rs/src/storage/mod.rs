// Storage domain

use crate::events::MarketDataEvent;
use crate::snapshots::MarketDataSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRecord {
    pub sequence: u64,
    pub event: MarketDataEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotRecord {
    pub sequence: u64,
    pub snapshot: MarketDataSnapshot,
}
