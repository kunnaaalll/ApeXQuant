use crate::shadow::comparison::{ShadowComparisonResult, ShadowComparisonState};
use crate::shadow::drift::DriftMeasurement;
use crate::shadow::statistics::ShadowStatistics;
use rust_decimal::Decimal;
use std::collections::HashMap;

pub struct ShadowValidator {
    pub statistics: ShadowStatistics,
    pub tolerance: Decimal,
    pub critical_threshold: Decimal,
    pub active_drifts: HashMap<String, Vec<DriftMeasurement>>,
}

impl ShadowValidator {
    pub fn new(tolerance: Decimal, critical_threshold: Decimal) -> Self {
        Self {
            statistics: ShadowStatistics::new(),
            tolerance,
            critical_threshold,
            active_drifts: HashMap::new(),
        }
    }

    pub fn validate_execution(
        &mut self,
        identifier: &str,
        legacy_metrics: HashMap<String, Decimal>,
        rust_metrics: HashMap<String, Decimal>,
    ) -> ShadowComparisonResult {
        let mut max_diff = Decimal::ZERO;
        let mut mismatch_count = 0;
        let mut total_metrics = 0;

        let mut drifts = Vec::new();

        // Compare all metrics provided
        for (metric_name, legacy_val) in legacy_metrics.iter() {
            total_metrics += 1;

            let rust_val = rust_metrics
                .get(metric_name)
                .copied()
                .unwrap_or(Decimal::ZERO);

            let drift = DriftMeasurement::new(metric_name.clone(), *legacy_val, rust_val);

            if drift.absolute_drift > max_diff {
                max_diff = drift.absolute_drift;
            }

            if drift.has_significant_drift(self.tolerance) {
                mismatch_count += 1;
            }

            drifts.push(drift);
        }

        // Store drifts for later analysis
        self.active_drifts.insert(identifier.to_string(), drifts);

        let agreement_percentage = if total_metrics == 0 {
            Decimal::new(100, 0)
        } else {
            let matched = total_metrics - mismatch_count;
            (Decimal::from(matched) / Decimal::from(total_metrics)) * Decimal::new(100, 0)
        };

        let mut result = ShadowComparisonResult {
            agreement_percentage: agreement_percentage.round_dp(4),
            mismatch_count: mismatch_count as u64,
            average_difference: if total_metrics > 0 {
                max_diff / Decimal::from(total_metrics)
            } else {
                Decimal::ZERO
            },
            maximum_difference: max_diff,
            state: ShadowComparisonState::ExactMatch,
        };

        result.determine_state(self.tolerance, self.critical_threshold);

        // Record into global stats
        self.statistics.record_result(result.state, max_diff);

        result
    }

    pub fn is_parity_achieved(&self, target_agreement: Decimal) -> bool {
        self.statistics.get_overall_agreement_percentage() >= target_agreement
    }
}
