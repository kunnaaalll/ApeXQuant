use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeState {
    Accelerating,
    Improving,
    Stable,
    Weakening,
    Critical,
}

#[derive(Debug, Clone)]
pub struct EdgeIntelligence {
    pub raw_edge: Decimal,
    pub recent_edge: Decimal,
    pub long_term_edge: Decimal,
    pub edge_difference: Decimal,
    pub improving: bool,
    pub degrading: bool,
    pub stable: bool,
    pub edge_state: EdgeState,
}

impl EdgeIntelligence {
    pub fn evaluate(recent_edge: Decimal, long_term_edge: Decimal) -> Self {
        let edge_difference = recent_edge - long_term_edge;
        
        let margin = (long_term_edge.abs() * dec!(0.05)).max(dec!(0.01));
        
        let improving = edge_difference > margin;
        let degrading = edge_difference < -margin;
        let stable = edge_difference.abs() <= margin;

        let edge_state = if edge_difference < -dec!(0.2) {
            EdgeState::Critical
        } else if edge_difference < -margin {
            EdgeState::Weakening
        } else if stable {
            EdgeState::Stable
        } else if edge_difference > dec!(0.2) {
            EdgeState::Accelerating
        } else {
            EdgeState::Improving
        };

        Self {
            raw_edge: recent_edge,
            recent_edge,
            long_term_edge,
            edge_difference,
            improving,
            degrading,
            stable,
            edge_state,
        }
    }
}
