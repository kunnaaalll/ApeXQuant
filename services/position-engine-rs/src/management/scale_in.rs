//! Dynamic Scale-In Engine
//!
//! Computes additional position size based on account risk parameters.
//! Replaces the static hardcoded `10 units` placeholder.
//!
//! Formula:
//! ```text
//! available_risk = min(remaining_daily_loss, max_risk_allowed − current_risk)
//! exposure_headroom = 1 − (current_exposure / max_exposure)
//! atr_position = available_risk / distance_to_stop
//! scale_in_size = atr_position × exposure_headroom × volatility_scalar
//! volatility_scalar = target_volatility / expected_volatility  (clipped 0.5–1.0)
//! ```

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::health::PositionQuality;

/// All inputs required for dynamic scale-in sizing.
#[derive(Debug, Clone)]
pub struct ScaleInInputs {
    /// Current account balance in account currency
    pub account_balance: Decimal,
    /// Average True Range in account currency per unit
    pub atr: Decimal,
    /// Current total risk exposure (sum of all open position risks)
    pub current_risk: Decimal,
    /// Maximum total risk permitted at any time
    pub max_risk_allowed: Decimal,
    /// Distance from current price to stop loss in account currency per unit
    pub distance_to_stop: Decimal,
    /// Current open exposure as fraction of account (0–1)
    pub current_exposure_fraction: Decimal,
    /// Maximum allowed exposure fraction (0–1)
    pub max_exposure_fraction: Decimal,
    /// Expected volatility of the instrument (annualised, 0–1)
    pub expected_volatility: Decimal,
    /// Target portfolio volatility (annualised, 0–1)
    pub target_volatility: Decimal,
    /// Remaining daily loss budget in account currency
    pub remaining_daily_loss: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleInRecommendation {
    pub should_scale: bool,
    pub additional_size: Decimal,
    pub holding_efficiency: Decimal,
    pub reason: String,
}

pub struct ScaleInEngine;

impl ScaleInEngine {
    /// Evaluate whether to scale into a position and how much.
    pub fn evaluate(quality: &PositionQuality, inputs: &ScaleInInputs) -> ScaleInRecommendation {
        // Gate 1: quality must be excellent
        if *quality != PositionQuality::Excellent {
            return ScaleInRecommendation {
                should_scale: false,
                additional_size: Decimal::ZERO,
                holding_efficiency: Decimal::ZERO,
                reason: format!("Quality is {:?}; scaling in requires Excellent", quality),
            };
        }

        // Gate 2: must have remaining risk budget
        if inputs.current_risk >= inputs.max_risk_allowed {
            return ScaleInRecommendation {
                should_scale: false,
                additional_size: Decimal::ZERO,
                holding_efficiency: Decimal::ZERO,
                reason: "Maximum risk exposure reached; cannot scale in".to_string(),
            };
        }

        // Gate 3: must have remaining daily loss budget
        if inputs.remaining_daily_loss <= Decimal::ZERO {
            return ScaleInRecommendation {
                should_scale: false,
                additional_size: Decimal::ZERO,
                holding_efficiency: Decimal::ZERO,
                reason: "Daily loss budget exhausted; scaling blocked".to_string(),
            };
        }

        // Gate 4: distance to stop must be positive
        if inputs.distance_to_stop <= Decimal::ZERO {
            return ScaleInRecommendation {
                should_scale: false,
                additional_size: Decimal::ZERO,
                holding_efficiency: Decimal::ZERO,
                reason: "Distance to stop is zero or negative; cannot compute position size"
                    .to_string(),
            };
        }

        // --- Dynamic sizing formula ---

        // Available risk = min(remaining_daily_loss, risk_headroom)
        let risk_headroom = inputs.max_risk_allowed - inputs.current_risk;
        let available_risk = inputs.remaining_daily_loss.min(risk_headroom);

        // Exposure headroom: how much room remains before hitting max exposure
        let exposure_headroom = if inputs.max_exposure_fraction > Decimal::ZERO {
            let used_fraction = inputs.current_exposure_fraction / inputs.max_exposure_fraction;
            (Decimal::ONE - used_fraction).max(Decimal::ZERO)
        } else {
            Decimal::ZERO
        };

        if exposure_headroom <= Decimal::ZERO {
            return ScaleInRecommendation {
                should_scale: false,
                additional_size: Decimal::ZERO,
                holding_efficiency: Decimal::ZERO,
                reason: "Maximum exposure fraction reached; cannot scale in".to_string(),
            };
        }

        // Base position size from risk/stop distance
        let atr_position = available_risk / inputs.distance_to_stop;

        // Volatility scalar: reduce size when expected vol exceeds target
        let volatility_scalar = if inputs.expected_volatility > Decimal::ZERO
            && inputs.target_volatility > Decimal::ZERO
        {
            let raw = inputs.target_volatility / inputs.expected_volatility;
            raw.max(Decimal::new(5, 1)).min(Decimal::ONE) // clip [0.5, 1.0]
        } else {
            Decimal::ONE
        };

        // Final scale-in size
        let additional_size = (atr_position * exposure_headroom * volatility_scalar).round_dp(4); // round to 4dp (standard lot precision)

        if additional_size <= Decimal::ZERO {
            return ScaleInRecommendation {
                should_scale: false,
                additional_size: Decimal::ZERO,
                holding_efficiency: Decimal::ZERO,
                reason: "Computed scale-in size is zero or negative after constraints".to_string(),
            };
        }

        // Holding efficiency: ratio of available risk to account balance
        // Higher = more efficient use of capital per unit of exposure
        let holding_efficiency = if inputs.account_balance > Decimal::ZERO {
            (available_risk / inputs.account_balance).round_dp(4)
        } else {
            Decimal::ZERO
        };

        ScaleInRecommendation {
            should_scale: true,
            additional_size,
            holding_efficiency,
            reason: format!(
                "Scale-in: available_risk={:.2} distance_to_stop={:.5} exposure_headroom={:.2} vol_scalar={:.2} → size={:.4}",
                available_risk, inputs.distance_to_stop, exposure_headroom, volatility_scalar, additional_size
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_inputs() -> ScaleInInputs {
        ScaleInInputs {
            account_balance: Decimal::new(100_000, 0),
            atr: Decimal::new(50, 0),
            current_risk: Decimal::new(500, 0),
            max_risk_allowed: Decimal::new(2_000, 0),
            distance_to_stop: Decimal::new(100, 0),
            current_exposure_fraction: Decimal::new(30, 2),
            max_exposure_fraction: Decimal::new(80, 2),
            expected_volatility: Decimal::new(15, 2),
            target_volatility: Decimal::new(15, 2),
            remaining_daily_loss: Decimal::new(1_000, 0),
        }
    }

    #[test]
    fn test_excellent_quality_scales() {
        let inputs = default_inputs();
        let result = ScaleInEngine::evaluate(&PositionQuality::Excellent, &inputs);
        assert!(result.should_scale);
        assert!(result.additional_size > Decimal::ZERO);
    }

    #[test]
    fn test_non_excellent_blocked() {
        let inputs = default_inputs();
        let result = ScaleInEngine::evaluate(&PositionQuality::Weak, &inputs);
        assert!(!result.should_scale);
        assert_eq!(result.additional_size, Decimal::ZERO);
    }

    #[test]
    fn test_max_risk_blocks() {
        let mut inputs = default_inputs();
        inputs.current_risk = inputs.max_risk_allowed;
        let result = ScaleInEngine::evaluate(&PositionQuality::Excellent, &inputs);
        assert!(!result.should_scale);
    }

    #[test]
    fn test_no_daily_budget_blocks() {
        let mut inputs = default_inputs();
        inputs.remaining_daily_loss = Decimal::ZERO;
        let result = ScaleInEngine::evaluate(&PositionQuality::Excellent, &inputs);
        assert!(!result.should_scale);
    }

    #[test]
    fn test_high_volatility_reduces_size() {
        let normal = default_inputs();
        let mut high_vol = default_inputs();
        high_vol.expected_volatility = Decimal::new(40, 2); // Double the target

        let normal_result = ScaleInEngine::evaluate(&PositionQuality::Excellent, &normal);
        let high_vol_result = ScaleInEngine::evaluate(&PositionQuality::Excellent, &high_vol);

        // High vol should reduce size (volatility scalar < 1)
        assert!(high_vol_result.additional_size <= normal_result.additional_size);
    }
}
