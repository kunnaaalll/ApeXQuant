use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use super::models::{DegradationAssessment, DegradationMetrics};
use super::states::DegradationState;

pub struct DegradationDetector;

impl DegradationDetector {
    pub fn detect(
        historical_edge: Decimal,
        current_edge: Decimal,
        historical_expectancy: Decimal,
        current_expectancy: Decimal,
        historical_stability: Decimal,
        current_stability: Decimal,
        historical_quality: Decimal,
        current_quality: Decimal,
        duration: u32,
    ) -> DegradationAssessment {
        let edge_decay = historical_edge - current_edge;
        let expectancy_decay = historical_expectancy - current_expectancy;
        let stability_deterioration = historical_stability - current_stability;
        let quality_deterioration = historical_quality - current_quality;
        
        // Simple heuristic for performance drift
        let performance_drift = if historical_expectancy.is_zero() {
            Decimal::ZERO
        } else {
            (current_expectancy - historical_expectancy) / historical_expectancy.abs()
        };

        // If drift is positive, severity is 0
        let severity = if performance_drift < Decimal::ZERO {
            performance_drift.abs()
        } else {
            Decimal::ZERO
        };

        let duration_dec = Decimal::from(duration);
        let velocity = if duration > 0 {
            severity / duration_dec
        } else {
            Decimal::ZERO
        };

        let metrics = DegradationMetrics {
            edge_decay: edge_decay.max(Decimal::ZERO), // Only track decay, not improvement here
            expectancy_decay: expectancy_decay.max(Decimal::ZERO),
            quality_deterioration: quality_deterioration.max(Decimal::ZERO),
            stability_deterioration: stability_deterioration.max(Decimal::ZERO),
            performance_drift,
            duration,
            severity,
            velocity,
        };

        let state = Self::determine_state(&metrics);

        DegradationAssessment {
            metrics,
            state,
        }
    }

    pub fn determine_state(metrics: &DegradationMetrics) -> DegradationState {
        if metrics.severity > dec!(0.5) && metrics.velocity > dec!(0.1) {
            DegradationState::Critical
        } else if metrics.severity > dec!(0.3) {
            DegradationState::Warning
        } else if metrics.severity > dec!(0.1) {
            DegradationState::Watch
        } else {
            DegradationState::Healthy
        }
    }
}
