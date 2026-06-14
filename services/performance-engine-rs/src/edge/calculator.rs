use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use super::models::{EdgeAssessment, EdgeMetrics};
use super::states::EdgeState;

pub struct EdgeCalculator;

impl EdgeCalculator {
    pub fn calculate(
        _expectancy: Decimal,
        win_rate: Decimal,
        average_rr: Decimal,
        trade_count: u32,
    ) -> EdgeAssessment {
        let trade_count_dec = Decimal::from(trade_count);
        
        let raw_edge = (win_rate * average_rr) - (dec!(1.0) - win_rate);
        
        // Edge score mapping: logic to map raw_edge and expectancy into a 0-100 score.
        // Bounded mathematically.
        let mut edge_score = if raw_edge > dec!(0.0) {
            (raw_edge * dec!(100.0)).min(dec!(100.0))
        } else {
            Decimal::ZERO
        };

        // Confidence modifier based on sample size
        let edge_confidence = if trade_count < 30 {
            trade_count_dec / dec!(30.0)
        } else {
            dec!(1.0)
        };

        edge_score = edge_score * edge_confidence;

        let state = Self::determine_state(edge_score);

        let metrics = EdgeMetrics {
            raw_edge,
            edge_score,
            edge_acceleration: Decimal::ZERO, // Computed in degradation/velocity engines usually
            edge_decay: Decimal::ZERO,
            edge_confidence,
            edge_stability: Decimal::ZERO,
        };

        EdgeAssessment {
            metrics,
            state,
        }
    }

    pub fn determine_state(edge_score: Decimal) -> EdgeState {
        if edge_score > dec!(80.0) {
            EdgeState::HighEdge
        } else if edge_score > dec!(50.0) {
            EdgeState::MediumEdge
        } else if edge_score > dec!(20.0) {
            EdgeState::LowEdge
        } else {
            EdgeState::NoEdge
        }
    }
}
