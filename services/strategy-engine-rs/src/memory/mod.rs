use rust_decimal::Decimal;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfidenceMemory {
    pub historical_confidence: VecDeque<Decimal>,
    pub degradation_history: VecDeque<Decimal>,
    pub edge_history: VecDeque<Decimal>,
    pub capacity: usize,
}

impl ConfidenceMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            historical_confidence: VecDeque::with_capacity(capacity),
            degradation_history: VecDeque::with_capacity(capacity),
            edge_history: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn record(&mut self, confidence: Decimal, degradation: Decimal, edge: Decimal) {
        if self.capacity == 0 {
            return;
        }

        if self.historical_confidence.len() == self.capacity {
            self.historical_confidence.pop_back();
            self.degradation_history.pop_back();
            self.edge_history.pop_back();
        }

        self.historical_confidence.push_front(confidence);
        self.degradation_history.push_front(degradation);
        self.edge_history.push_front(edge);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeMemory {
    pub rolling_edge_history: VecDeque<Decimal>,
    pub rolling_expectancy_history: VecDeque<Decimal>,
    pub capacity: usize,
}

impl EdgeMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            rolling_edge_history: VecDeque::with_capacity(capacity),
            rolling_expectancy_history: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn record(&mut self, edge: Decimal, expectancy: Decimal) {
        if self.capacity == 0 {
            return;
        }

        if self.rolling_edge_history.len() == self.capacity {
            self.rolling_edge_history.pop_back();
            self.rolling_expectancy_history.pop_back();
        }

        self.rolling_edge_history.push_front(edge);
        self.rolling_expectancy_history.push_front(expectancy);
    }
}
