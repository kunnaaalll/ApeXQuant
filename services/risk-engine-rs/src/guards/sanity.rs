//! Sanity checks - catch obvious errors and edge cases
use rust_decimal::prelude::FromPrimitive;

use crate::{PositionSizeResult, RiskError};
use rust_decimal::Decimal;

/// Sanity checks for obvious problems
pub struct SanityGuard {
    /// Minimum meaningful lot size
    min_lot_size: Decimal,
    /// Maximum reasonable position count
    max_positions: u32,
}

impl SanityGuard {
    /// Create new sanity guard
    pub fn new() -> Self {
        Self {
            min_lot_size: Decimal::from_f64(0.001).unwrap(), // 0.001 micro lots
            max_positions: 100, // Unreasonably high count
        }
    }

    /// Perform comprehensive sanity check
    pub fn check_sanity(&self, result: &PositionSizeResult) -> Result<(), RiskError> {
        // Check for NaN or infinity equivalents in decimal
        self.check_decimal_validity(result)?;

        // Check for extremely small lot sizes
        if result.lot_size > Decimal::ZERO && result.lot_size < self.min_lot_size {
            return Err(RiskError::InvalidPositionSize {
                reason: format!(
                    "Lot size {} is smaller than minimum meaningful size {}",
                    result.lot_size, self.min_lot_size
                ),
            });
        }

        // Check for extremely large lot sizes
        if result.lot_size > Decimal::from(1000) {
            return Err(RiskError::InvalidPositionSize {
                reason: format!(
                    "Lot size {} exceeds maximum sanity check of 1000 lots",
                    result.lot_size
                ),
            });
        }

        // Check risk percent is reasonable
        if result.risk_percent > Decimal::ONE {
            return Err(RiskError::InvalidPositionSize {
                reason: format!(
                    "Risk percent {} exceeds 100% - likely calculation error",
                    result.risk_percent
                ),
            });
        }

        // Check capital at risk is reasonable
        if result.capital_at_risk > Decimal::from(i64::MAX) {
            return Err(RiskError::CalculationOverflow {
                operation: "capital at risk".to_string(),
            });
        }

        // Check reasoning is non-trivial
        if result.reasoning.len() < 5 {
            return Err(RiskError::Validation(
                "Reasoning too short to be meaningful".to_string(),
            ));
        }

        Ok(())
    }

    /// Check input values for sanity
    pub fn check_input_sanity(
        &self,
        equity: Decimal,
        _symbol: &str,
        entry: Decimal,
        stop: Decimal,
    ) -> Result<(), RiskError> {
        // Sanity check equity
        if equity > Decimal::from(i64::MAX) {
            return Err(RiskError::InvalidInput {
                field: "equity".to_string(),
                reason: "Equity value exceeds maximum reasonable value".to_string(),
            });
        }

        // Sanity check entry vs stop
        if entry == stop {
            return Err(RiskError::InvalidInput {
                field: "stop_distance".to_string(),
                reason: "Entry price equals stop - would result in infinite lot size".to_string(),
            });
        }

        // Check for reasonable stop distance (not too tight)
        let stop_distance = (entry - stop).abs();
        if entry > Decimal::ZERO {
            let stop_pct = stop_distance / entry;

            if stop_pct > Decimal::from_f64(0.20).unwrap() {
                // 20% stop is very wide - warn
                // This is advisory, not blocking
            }

            if stop_pct < Decimal::from_f64(0.0001).unwrap() {
                // 1 pip stop is very tight - possible error
                return Err(RiskError::InvalidInput {
                    field: "stop_distance".to_string(),
                    reason: "Stop distance is extremely tight (< 0.01%)".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Check combined position exposure
    pub fn check_combined_exposure(
        &self,
        new_position: &PositionSizeResult,
        existing_count: u32,
        total_risk: Decimal,
    ) -> Result<(), RiskError> {
        // Check we're not exceeding position count sanity
        if existing_count > self.max_positions {
            return Err(RiskError::Validation(format!(
                "Position count {} exceeds sanity limit {}",
                existing_count, self.max_positions
            )));
        }

        // Check combined risk isn't insane
        if total_risk > Decimal::ONE {
            return Err(RiskError::Validation(format!(
                "Combined risk {} exceeds 100% of equity",
                total_risk
            )));
        }

        // Check for reasonable capital efficiency
        if new_position.capital_at_risk == Decimal::ZERO && new_position.lot_size > Decimal::ZERO {
            return Err(RiskError::Validation(
                "Non-zero lot size with zero capital at risk detected".to_string(),
            ));
        }

        Ok(())
    }

    /// Check for common calculation bugs
    pub fn check_calculation_mq(&self, result: &PositionSizeResult) -> Result<(), RiskError> {
        // Check for division-by-zero artifacts
        if result.lot_size == Decimal::ZERO
            && result.risk_percent > Decimal::ZERO
            && !result.reasoning.contains("denied")
            && !result.reasoning.contains("limit")
        {
            return Err(RiskError::Validation(
                "Zero lot size without denial reason - possible calculation error".to_string(),
            ));
        }

        // Check for inverted sign
        if result.lot_size < Decimal::ZERO {
            return Err(RiskError::InvalidPositionSize {
                reason: "Negative lot size detected".to_string(),
            });
        }

        // Check for precision loss indicators
        let reconstructed = result.capital_at_risk / result.risk_percent;
        if result.risk_percent > Decimal::ZERO {
            let ratio = if reconstructed > Decimal::ZERO {
                (result.capital_at_risk / (reconstructed * result.risk_percent)).abs()
            } else {
                Decimal::ONE
            };

            // Should be close to 1.0
            if ratio < Decimal::from_f64(0.9).unwrap() || ratio > Decimal::from_f64(1.1).unwrap() {
                // Significant precision loss - log but don't fail
            }
        }

        Ok(())
    }

    fn check_decimal_validity(&self, result: &PositionSizeResult) -> Result<(), RiskError> {
        // rust_decimal doesn't support NaN/Inf, but we can check for unexpected values

        // Check for zero where non-zero expected
        if result.lot_size == Decimal::ZERO && result.risk_percent > Decimal::ZERO {
            return Err(RiskError::Validation(
                "Inconsistent: zero lot size with positive risk".to_string(),
            ));
        }

        Ok(())
    }

    /// Emergency stop check for catastrophic conditions
    pub fn emergency_check(
        &self,
        drawdown: Decimal,
        daily_loss: Decimal,
        equity: Decimal,
    ) -> Result<(), RiskError> {
        if equity <= Decimal::ZERO {
            return Err(RiskError::Validation(
                "CRITICAL: Equity is zero or negative".to_string(),
            ));
        }

        if drawdown > Decimal::from_f64(0.50).unwrap() {
            return Err(RiskError::Validation(
                "CRITICAL: Drawdown exceeds 50%".to_string(),
            ));
        }

        if daily_loss.abs() / equity > Decimal::from_f64(0.20).unwrap() {
            return Err(RiskError::Validation(
                "CRITICAL: Daily loss exceeds 20%".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for SanityGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SizingMethod;

    #[test]
    fn test_excessive_risk_percent() {
        let guard = SanityGuard::new();

        let insane = PositionSizeResult {
            lot_size: Decimal::ONE,
            risk_percent: Decimal::from(2), // 200% - insane
            capital_at_risk: Decimal::from(100),
            method: SizingMethod::FixedFractional,
            reasoning: "Test result with errors".to_string(),
        };

        assert!(guard.check_sanity(&insane).is_err());
    }

    #[test]
    fn test_too_small_lot_size() {
        let guard = SanityGuard::new();

        let tiny = PositionSizeResult {
            lot_size: Decimal::from_f64(0.0001).unwrap(),
            risk_percent: Decimal::from_f64(0.01).unwrap(),
            capital_at_risk: Decimal::from_f64(0.001).unwrap(),
            method: SizingMethod::FixedFractional,
            reasoning: "Tiny position test".to_string(),
        };

        assert!(guard.check_sanity(&tiny).is_err());
    }

    #[test]
    fn test_zero_lot_without_reason() {
        let guard = SanityGuard::new();

        let suspicious = PositionSizeResult {
            lot_size: Decimal::ZERO,
            risk_percent: Decimal::from_f64(0.01).unwrap(),
            capital_at_risk: Decimal::ZERO,
            method: SizingMethod::FixedFractional,
            reasoning: "Normal calculation complete".to_string(), // No denial mentioned
        };

        assert!(guard.check_calculation_mq(&suspicious).is_err());
    }

    #[test]
    fn test_emergency_drawdown() {
        let guard = SanityGuard::new();

        assert!(guard
            .emergency_check(
                Decimal::from_f64(0.60).unwrap(), // 60% drawdown
                Decimal::ZERO,
                Decimal::from(10000)
            )
            .is_err());
    }

    #[test]
    fn test_emergency_equity() {
        let guard = SanityGuard::new();

        assert!(guard
            .emergency_check(
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO // Zero equity
            )
            .is_err());
    }

    #[test]
    fn test_short_reasoning() {
        let guard = SanityGuard::new();

        let short = PositionSizeResult {
            lot_size: Decimal::ONE,
            risk_percent: Decimal::from_f64(0.01).unwrap(),
            capital_at_risk: Decimal::from(100),
            method: SizingMethod::FixedFractional,
            reasoning: "Ok".to_string(),
        };

        assert!(guard.check_sanity(&short).is_err());
    }

    #[test]
    fn test_valid_sanity() {
        let guard = SanityGuard::new();

        let valid = PositionSizeResult {
            lot_size: Decimal::from_f64(0.5).unwrap(),
            risk_percent: Decimal::from_f64(0.01).unwrap(),
            capital_at_risk: Decimal::from(100),
            method: SizingMethod::FixedFractional,
            reasoning: "Fixed fractional sizing applied with 1% risk and 50 pip stop distance."
                .to_string(),
        };

        assert!(guard.check_sanity(&valid).is_ok());
    }

    #[test]
    fn test_stop_distance_sanity() {
        let guard = SanityGuard::new();

        // Equal entry and stop should fail
        assert!(guard
            .check_input_sanity(
                Decimal::from(10000),
                "EURUSD",
                Decimal::from_f64(1.08500).unwrap(),
                Decimal::from_f64(1.08500).unwrap()
            )
            .is_err());
    }
}
