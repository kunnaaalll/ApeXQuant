use super::{ClusterState, ClusterType};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterEngine {
    state: ClusterState,
}

impl ClusterEngine {
    pub fn new() -> Self {
        Self {
            state: ClusterState::new(),
        }
    }

    pub fn update(&mut self, cluster: ClusterType, raw_confidence: Decimal) {
        let min_confidence = dec!(0.0);
        let max_confidence = dec!(100.0);

        let confidence = if raw_confidence < min_confidence {
            min_confidence
        } else if raw_confidence > max_confidence {
            max_confidence
        } else {
            raw_confidence
        };

        self.state.active_cluster = cluster;
        self.state.confidence = confidence;
    }

    pub fn state(&self) -> &ClusterState {
        &self.state
    }
}

impl Default for ClusterEngine {
    fn default() -> Self {
        Self::new()
    }
}
