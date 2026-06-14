use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Applied as a multiplicative penalty to confidence scores.
/// Combined from multiple signals — never additive, always multiplicative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidencePenaltyReport {
    /// Sample-size penalty factor [0, 1]
    pub sample_penalty: Decimal,
    /// Parameter sensitivity penalty [0, 1]: fragile edge = high sensitivity
    pub sensitivity_penalty: Decimal,
    /// Out-of-sample vs in-sample performance ratio penalty [0, 1]
    pub oos_penalty: Decimal,
    /// Final combined penalty [0, 1] (product of all)
    pub combined_penalty: Decimal,
    pub explanation: String,
}

pub struct ConfidencePenaltyEngine;

impl ConfidencePenaltyEngine {
    /// All penalties are in [0, 1] — multiply by base confidence to get adjusted confidence.
    /// `sample_penalty`     : from SampleBiasDetector
    /// `sensitivity_ratio`  : measured parameter sensitivity [0, 1] (1 = highly fragile)
    /// `oos_ratio`          : out-of-sample performance / in-sample performance [0, 1]
    pub fn compute(
        sample_penalty: Decimal,
        sensitivity_ratio: Decimal,
        oos_ratio: Decimal,
    ) -> ConfidencePenaltyReport {
        // Clamp inputs
        let sp = sample_penalty.clamp(dec!(0), dec!(1));
        // High sensitivity means low confidence — invert
        let sens_penalty = (dec!(1) - sensitivity_ratio).clamp(dec!(0), dec!(1));
        // OOS near 1 = strong, near 0 = overfit
        let oos_p = oos_ratio.clamp(dec!(0), dec!(1));

        let combined_penalty = sp * sens_penalty * oos_p;

        let explanation = format!(
            "Confidence penalty breakdown — sample: {:.3}, sensitivity: {:.3}, OOS: {:.3}, combined: {:.3}",
            sp, sens_penalty, oos_p, combined_penalty
        );

        ConfidencePenaltyReport {
            sample_penalty: sp,
            sensitivity_penalty: sens_penalty,
            oos_penalty: oos_p,
            combined_penalty,
            explanation,
        }
    }

    /// Apply penalty to a raw confidence score.
    pub fn apply(raw_confidence: Decimal, penalty: &ConfidencePenaltyReport) -> Decimal {
        (raw_confidence * penalty.combined_penalty).clamp(dec!(0), dec!(1))
    }
}
