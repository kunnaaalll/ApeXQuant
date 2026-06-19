use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeState {
    Emerging,
    Strengthening,
    Stable,
    Weakening,
    Collapsing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeDiscovery {
    state: EdgeState,
}

impl EdgeDiscovery {
    pub fn new() -> Self {
        Self {
            state: EdgeState::Stable,
        }
    }

    pub fn detect(
        &mut self,
        edge_delta: Decimal,
        expectancy_delta: Decimal,
        confidence_delta: Decimal,
    ) -> EdgeState {
        let combined_delta = edge_delta + expectancy_delta + confidence_delta;

        let threshold_strong = dec!(0.10);
        let threshold_emerge = dec!(0.03);
        let threshold_weak = dec!(-0.03);
        let threshold_collapse = dec!(-0.10);

        self.state = if combined_delta >= threshold_strong {
            EdgeState::Strengthening
        } else if combined_delta > threshold_emerge {
            EdgeState::Emerging
        } else if combined_delta <= threshold_collapse {
            EdgeState::Collapsing
        } else if combined_delta < threshold_weak {
            EdgeState::Weakening
        } else {
            EdgeState::Stable
        };

        self.state
    }

    pub fn state(&self) -> EdgeState {
        self.state
    }
}

impl Default for EdgeDiscovery {
    fn default() -> Self {
        Self::new()
    }
}
