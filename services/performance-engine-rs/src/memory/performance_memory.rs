use rust_decimal::Decimal;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    pub timestamp: u64,
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub stability: Decimal,
}

#[derive(Debug, Clone)]
pub struct PerformanceMemory {
    pub max_snapshots: usize,
    pub snapshots: VecDeque<PerformanceSnapshot>,
}

impl PerformanceMemory {
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            max_snapshots,
            snapshots: VecDeque::with_capacity(max_snapshots),
        }
    }

    pub fn record_snapshot(&mut self, snapshot: PerformanceSnapshot) {
        if self.snapshots.len() >= self.max_snapshots {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(snapshot);
    }
}
