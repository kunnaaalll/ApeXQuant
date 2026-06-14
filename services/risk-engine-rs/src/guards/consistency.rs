//! Internal consistency checks
use rust_decimal::prelude::FromPrimitive;

use crate::{PositionSizeResult, RiskError};
use rust_decimal::Decimal;

/// Checks internal consistency of risk calculations
pub struct ConsistencyGuard;

impl ConsistencyGuard {
    /// Create new consistency guard
    pub fn new() -> Self {
        Self
    }

    /// Check result for internal consistency
    pub fn check_result(&self, result: &PositionSizeResult) -> Result<(), RiskError> {
        // Verify risk_percent * equity ≈ capital_at_risk
        // Note: We don't have equity here, so we check proportionality
        if result.lot_size > Decimal::ZERO {
            // Check that all components are proportional
            self.check_proportionality(result)?;
        }

        // Verify risk percent is within reasonable bounds given method
        self.check_method_consistency(result)?;

        // Check that reasoning is present
        if result.reasoning.is_empty() {
            return Err(RiskError::Validation("Missing reasoning".to_string()));
        }

        Ok(())
    }

    /// Verify capital at risk is proportional to lot size and risk percent
    fn check_proportionality(&self, result: &PositionSizeResult) -> Result<(), RiskError> {
        // Basic sanity: if lot_size increases, capital_at_risk should increase
        // And risk_percent should reflect the ratio

        // Check non-negativity
        if result.risk_percent < Decimal::ZERO {
            return Err(RiskError::Validation("Negative risk percent".to_string()));
        }

        if result.capital_at_risk < Decimal::ZERO {
            return Err(RiskError::Validation("Negative capital at risk".to_string()));
        }

        // Check that risk_percent is reasonably sized
        // Max 10% risk per trade is extreme, should catch calculation errors
        if result.risk_percent > Decimal::from_f64(0.10).unwrap() {
            return Err(RiskError::Validation(format!(
                "Risk percent {} exceeds reasonable maximum",
                result.risk_percent
            )));
        }

        // Check that capital_at_risk is proportional to risk_percent
        // We expect: capital_at_risk = equity * risk_percent
        // Without equity, we check that they're both zero or both non-zero
        if result.risk_percent == Decimal::ZERO && result.capital_at_risk > Decimal::ZERO {
            return Err(RiskError::Validation(
                "Inconsistent: zero risk percent with non-zero capital at risk".to_string(),
            ));
        }

        if result.risk_percent > Decimal::ZERO && result.capital_at_risk == Decimal::ZERO {
            return Err(RiskError::Validation(
                "Inconsistent: positive risk percent with zero capital at risk".to_string(),
            ));
        }

        Ok(())
    }

    /// Check consistency between sizing method and result
    fn check_method_consistency(&self, result: &PositionSizeResult) -> Result<(), RiskError> {
        use crate::position_sizing::SizingMethod;

        match result.method {
            SizingMethod::Minimal | SizingMethod::Conservative => {
                // These should produce smaller sizes
                if result.risk_percent > Decimal::from_f64(0.02).unwrap() {
                    return Err(RiskError::Validation(format!(
                        "Method {:?} produced excessive risk: {}",
                        result.method, result.risk_percent
                    )));
                }
            }
            SizingMethod::KellyFull => {
                // Full Kelly can be aggressive - just ensure it's not insane
                if result.risk_percent > Decimal::from_f64(0.20).unwrap() {
                    return Err(RiskError::Validation(
                        "Full Kelly produced excessive risk > 20%".to_string(),
                    ));
                }
            }
            SizingMethod::KellyFractional => {
                // Fractional Kelly should be moderate
                if result.risk_percent > Decimal::from_f64(0.10).unwrap() {
                    return Err(RiskError::Validation(
                        "Fractional Kelly produced risk > 10%".to_string(),
                    ));
                }
            }
            _ => {} // Other methods have wider acceptable ranges
        }

        Ok(())
    }

    /// Check that risk adjustments are in correct order
    pub fn check_adjustment_order(
        &self,
        base_risk: Decimal,
        adjustments: &[(String, Decimal)],
        final_risk: Decimal,
    ) -> Result<(), RiskError> {
        // Verify that adjustments were applied in reasonable order
        // Multiplicative adjustments should cascade properly

        let mut running = base_risk;
        for (name, adjustment) in adjustments {
            let new_running = running * *adjustment;

            // Check for calculation overflow
            if new_running > running * Decimal::from(1000) {
                return Err(RiskError::CalculationOverflow {
                    operation: format!("adjustment: {}", name),
                });
            }

            // Check for unreasonable reduction
            if *adjustment < Decimal::from_f64(0.001).unwrap() {
                return Err(RiskError::Validation(format!(
                    "Adjustment '{}' reduced risk by >99.9%: {}",
                    name, adjustment
                )));
            }

            running = new_running;
        }

        // Verify final matches expected (within rounding)
        let tolerance = final_risk * Decimal::from_f64(0.001).unwrap();
        if (running - final_risk).abs() > tolerance {
            return Err(RiskError::Validation(format!(
                "Adjustment calculation mismatch: expected {}, got {}",
                running, final_risk
            )));
        }

        Ok(())
    }

    /// Verify that correlation adjustments are reasonable
    pub fn check_correlation_consistency(
        &self,
        correlation_score: Decimal,
        position_size_reduction: Decimal,
    ) -> Result<(), RiskError> {
        // High correlation should lead to size reduction
        if correlation_score > Decimal::from_f64(0.8).unwrap()
            && position_size_reduction > Decimal::ONE
        {
            return Err(RiskError::Validation(
                "Inconsistent: high correlation should reduce, not increase, position size"
                    .to_string(),
            ));
        }

        // Verify reduction is proportional to correlation
        // Very rough check: >0.7 correlation should have >20% reduction
        if correlation_score > Decimal::from_f64(0.7).unwrap()
            && position_size_reduction > Decimal::from_f64(0.85).unwrap()
        {
            return Err(RiskError::Validation(format!(
                "Insufficient correlation reduction: score={}, reduction={}",
                correlation_score, position_size_reduction
            )));
        }

        Ok(())
    }

    /// Check drawdown state is consistent with position sizing
    pub fn check_drawdown_consistency(
        &self,
        drawdown_pct: Decimal,
        risk_percent: Decimal,
    ) -> Result<(), RiskError> {
        // In severe drawdown, risk should be reduced
        if drawdown_pct > Decimal::from_f64(0.05).unwrap()
            && risk_percent > Decimal::from_f64(0.015).unwrap()
        {
            // Warning condition - should reduce risk in drawdown
            // For now, just log - this is advisory
        }

        // Hard check: don't double down in drawdown
        if drawdown_pct > Decimal::from_f64(0.10).unwrap()
            && risk_percent > Decimal::from_f64(0.02).unwrap()
        {
            return Err(RiskError::Validation(
                "Excessive risk during drawdown > 10%".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for ConsistencyGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SizingMethod;

    #[test]
    fn test_zero_risk_consistency() {
        let guard = ConsistencyGuard::new();

        let result = PositionSizeResult {
            lot_size: Decimal::ONE,
            risk_percent: Decimal::ZERO,
            capital_at_risk: Decimal::from(100),
            method: SizingMethod::FixedFractional,
            reasoning: "Test".to_string(),
        };

        assert!(guard.check_result(&result).is_err());
    }

    #[test]
    fn test_method_consistency_kelly() {
        let guard = ConsistencyGuard::new();

        let excessive_kelly = PositionSizeResult {
            lot_size: Decimal::ONE,
            risk_percent: Decimal::from_f64(0.25).unwrap(), // Too high for Kelly
            capital_at_risk: Decimal::from(100),
            method: SizingMethod::KellyFractional,
            reasoning: "Test".to_string(),
        };

        assert!(guard.check_result(&excessive_kelly).is_err());
    }

    #[test]
    fn test_conservative_method_max_risk() {
        let guard = ConsistencyGuard::new();

        let excessive_conservative = PositionSizeResult {
            lot_size: Decimal::ONE,
            risk_percent: Decimal::from_f64(0.05).unwrap(), // Too high for conservative
            capital_at_risk: Decimal::from(100),
            method: SizingMethod::Conservative,
            reasoning: "Test".to_string(),
        };

        assert!(guard.check_result(&excessive_conservative).is_err());
    }

    #[test]
    fn test_adjustment_overflow() {
        let guard = ConsistencyGuard::new();

        let adjustments = vec![
            ("test".to_string(), Decimal::from(10000)),
        ];

        assert!(guard
            .check_adjustment_order(Decimal::ONE, &adjustments, Decimal::ZERO)
            .is_err());
    }

    #[test]
    fn test_correlation_consistency() {
        let guard = ConsistencyGuard::new();

        // High correlation with no reduction is wrong
        assert!(guard
            .check_correlation_consistency(
                Decimal::from_f64(0.9).unwrap(),
                Decimal::ONE // No reduction
            )
            .is_err());

        // High correlation with reduction is correct
        assert!(guard
            .check_correlation_consistency(
                Decimal::from_f64(0.9).unwrap(),
                Decimal::from_f64(0.5).unwrap() // 50% reduction
            )
            .is_ok());
    }

    #[test]
    fn test_drawdown_consistency() {
        let guard = ConsistencyGuard::new();

        // Severe drawdown with high risk is wrong
        assert!(guard
            .check_drawdown_consistency(
                Decimal::from_f64(0.15).unwrap(),
                Decimal::from_f64(0.03).unwrap()
            )
            .is_err());

        // Moderate drawdown with moderate risk is ok
        assert!(guard
            .check_drawdown_consistency(
                Decimal::from_f64(0.04).unwrap(),
                Decimal::from_f64(0.01).unwrap()
            )
            .is_ok());
    }

    #[test]
    fn test_valid_result() {
        let guard = ConsistencyGuard::new();

        let valid = PositionSizeResult {
            lot_size: Decimal::ONE,
            risk_percent: Decimal::from_f64(0.01).unwrap(),
            capital_at_risk: Decimal::from(100),
            method: SizingMethod::FixedFractional,
            reasoning: "Valid test result".to_string(),
        };

        assert!(guard.check_result(&valid).is_ok());
    }

    #[test]
    fn test_missing_reasoning() {
        let guard = ConsistencyGuard::new();

        let no_reason = PositionSizeResult {
            lot_size: Decimal::ONE,
            risk_percent: Decimal::from_f64(0.01).unwrap(),
            capital_at_risk: Decimal::from(100),
            method: SizingMethod::FixedFractional,
            reasoning: "".to_string(),
        };

        assert!(guard.check_result(&no_reason).is_err());
    }
}
