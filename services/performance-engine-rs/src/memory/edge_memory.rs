use rust_decimal::Decimal;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub value: Decimal,
    pub weight: Decimal,
}

#[derive(Debug, Clone)]
pub struct EdgeMemory {
    pub max_capacity: usize,
    pub history: VecDeque<MemoryEntry>,
    pub smoothed_value: Option<Decimal>,
}

impl EdgeMemory {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            max_capacity,
            history: VecDeque::with_capacity(max_capacity),
            smoothed_value: None,
        }
    }

    pub fn record(&mut self, value: Decimal, weight: Decimal, alpha: Decimal) {
        if self.history.len() >= self.max_capacity {
            self.history.pop_front();
        }
        self.history.push_back(MemoryEntry { value, weight });

        // Update smoothed value
        let one_minus_alpha = rust_decimal_macros::dec!(1.0) - alpha;
        self.smoothed_value = Some(match self.smoothed_value {
            Some(prev) => (value * alpha) + (prev * one_minus_alpha),
            None => value,
        });
    }
}
