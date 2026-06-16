use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorrelationWindowType {
    ShortTerm,
    MediumTerm,
    LongTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationWindow {
    pub window_type: CorrelationWindowType,
    pub active: bool,
    pub data_points: u64,
}

impl CorrelationWindow {
    pub fn new(window_type: CorrelationWindowType) -> Self {
        Self {
            window_type,
            active: true,
            data_points: 0,
        }
    }

    pub fn record_point(&mut self) {
        self.data_points = self.data_points.saturating_add(1);
    }
}
