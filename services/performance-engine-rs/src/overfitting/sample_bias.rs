use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Thresholds for sample adequacy — aligned with Phase 1 SampleQuality.
const MIN_INSTITUTIONAL: u32 = 300;
const MIN_STRONG: u32 = 100;
const MIN_ACCEPTABLE: u32 = 50;
const MIN_WEAK: u32 = 20;

/// Confidence penalty applied based on sample size deficit.
/// Returns a multiplier in [0.0, 1.0] — 1.0 means no penalty.
pub fn sample_size_penalty(trade_count: u32) -> Decimal {
    if trade_count >= MIN_INSTITUTIONAL {
        dec!(1.0)
    } else if trade_count >= MIN_STRONG {
        dec!(0.95)
    } else if trade_count >= MIN_ACCEPTABLE {
        dec!(0.80)
    } else if trade_count >= MIN_WEAK {
        dec!(0.50)
    } else {
        dec!(0.10)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleBiasReport {
    pub trade_count: u32,
    pub penalty_multiplier: Decimal,
    pub is_biased: bool,
    pub reason: String,
}

pub struct SampleBiasDetector;

impl SampleBiasDetector {
    pub fn evaluate(trade_count: u32) -> SampleBiasReport {
        let penalty_multiplier = sample_size_penalty(trade_count);
        let is_biased = trade_count < MIN_ACCEPTABLE;

        let reason = if trade_count < MIN_WEAK {
            format!("Critically insufficient sample ({} trades). Results are statistically meaningless.", trade_count)
        } else if trade_count < MIN_ACCEPTABLE {
            format!(
                "Weak sample ({} trades). High variance — results may not generalise.",
                trade_count
            )
        } else if trade_count < MIN_STRONG {
            format!(
                "Acceptable sample ({} trades). Moderate confidence only.",
                trade_count
            )
        } else if trade_count < MIN_INSTITUTIONAL {
            format!(
                "Strong sample ({} trades). Results are reliable.",
                trade_count
            )
        } else {
            format!(
                "Institutional sample ({} trades). Full confidence.",
                trade_count
            )
        };

        SampleBiasReport {
            trade_count,
            penalty_multiplier,
            is_biased,
            reason,
        }
    }
}
