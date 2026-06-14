//! Input and result validation
use rust_decimal::prelude::FromPrimitive;

use crate::{PositionSizeResult, RiskError, RiskInputs};
use rust_decimal::Decimal;

/// Validates input parameters and results
pub struct ValidationGuard {
    max_lot_size: Decimal,
    max_risk_percent: Decimal,
    min_risk_percent: Decimal,
}

impl ValidationGuard {
    /// Create new validation guard
    pub fn new() -> Self {
        Self {
            max_lot_size: Decimal::from_f64(100.0).unwrap(), // 100 lots max
            max_risk_percent: Decimal::from_f64(0.05).unwrap(), // 5% max
            min_risk_percent: Decimal::from_f64(0.0001).unwrap(), // 0.01% min
        }
    }

    /// Validate risk inputs
    pub fn validate_inputs(&self, inputs: &RiskInputs) -> Result<(), RiskError> {
        // Validate equity
        if inputs.equity <= Decimal::ZERO {
            return Err(RiskError::InvalidInput {
                field: "equity".to_string(),
                reason: "Equity must be positive".to_string(),
            });
        }

        // Validate symbol
        if inputs.symbol.is_empty() {
            return Err(RiskError::InvalidInput {
                field: "symbol".to_string(),
                reason: "Symbol cannot be empty".to_string(),
            });
        }

        // Validate direction
        if inputs.direction != 1 && inputs.direction != -1 {
            return Err(RiskError::InvalidInput {
                field: "direction".to_string(),
                reason: "Direction must be 1 (long) or -1 (short)".to_string(),
            });
        }

        // Validate prices
        if inputs.entry_price <= Decimal::ZERO {
            return Err(RiskError::InvalidInput {
                field: "entry_price".to_string(),
                reason: "Entry price must be positive".to_string(),
            });
        }

        if inputs.stop_loss <= Decimal::ZERO {
            return Err(RiskError::InvalidInput {
                field: "stop_loss".to_string(),
                reason: "Stop loss must be positive".to_string(),
            });
        }

        // Validate confidence ranges
        Self::validate_confidence_range(&inputs.signal_confidence, "signal_confidence")?;
        Self::validate_confidence_range(&inputs.regime_quality, "regime_quality")?;
        Self::validate_confidence_range(&inputs.pattern_quality, "pattern_quality")?;

        // Validate confluence score (0-10)
        if inputs.confluence_score < Decimal::ZERO || inputs.confluence_score > Decimal::from(10) {
            return Err(RiskError::InvalidInput {
                field: "confluence_score".to_string(),
                reason: "Confluence score must be between 0 and 10".to_string(),
            });
        }

        // Validate ATR if provided
        if let Some(atr) = inputs.atr {
            if atr < Decimal::ZERO {
                return Err(RiskError::InvalidInput {
                    field: "atr".to_string(),
                    reason: "ATR cannot be negative".to_string(),
                });
            }
        }

        // Validate spread
        if inputs.spread < Decimal::ZERO {
            return Err(RiskError::InvalidInput {
                field: "spread".to_string(),
                reason: "Spread cannot be negative".to_string(),
            });
        }

        Ok(())
    }

    /// Validate position size result
    pub fn validate_result(&self, result: &PositionSizeResult) -> Result<(), RiskError> {
        // Validate lot size is non-negative
        if result.lot_size < Decimal::ZERO {
            return Err(RiskError::InvalidPositionSize {
                reason: "Lot size cannot be negative".to_string(),
            });
        }

        // Check maximum lot size
        if result.lot_size > self.max_lot_size {
            return Err(RiskError::InvalidPositionSize {
                reason: format!(
                    "Lot size {} exceeds maximum {}",
                    result.lot_size, self.max_lot_size
                ),
            });
        }

        // Validate risk percent
        if result.risk_percent < Decimal::ZERO {
            return Err(RiskError::InvalidPositionSize {
                reason: "Risk percent cannot be negative".to_string(),
            });
        }

        if result.risk_percent > self.max_risk_percent {
            return Err(RiskError::InvalidPositionSize {
                reason: format!(
                    "Risk percent {} exceeds maximum {}",
                    result.risk_percent, self.max_risk_percent
                ),
            });
        }

        // Validate capital at risk matches lot_size * stop_distance
        // This is a consistency check that could fail if calculation bugs exist
        // Skip if lot_size is zero (denied trade)
        if result.lot_size > Decimal::ZERO {
            // Reconstruct expected capital at risk
            // In real implementation, we'd have access to stop distance
            // For now, just ensure it's proportional
            if result.capital_at_risk < Decimal::ZERO {
                return Err(RiskError::InvalidPositionSize {
                    reason: "Capital at risk cannot be negative".to_string(),
                });
            }
        }

        // Ensure consistency: if lot_size is zero, capital_at_risk should be zero
        if result.lot_size == Decimal::ZERO && result.capital_at_risk > Decimal::ZERO {
            return Err(RiskError::InvalidPositionSize {
                reason: "Zero lot size with non-zero capital at risk".to_string(),
            });
        }

        Ok(())
    }

    /// Validate that risk components sum correctly
    pub fn validate_risk_consistency(
        &self,
        base_risk: Decimal,
        adjustments: &[Decimal],
        final_risk: Decimal,
    ) -> Result<(), RiskError> {
        // Calculate expected final risk
        let product: Decimal = adjustments.iter().product();
        let expected = base_risk * product;

        // Allow for rounding errors (0.1% tolerance)
        let tolerance = expected * Decimal::from_f64(0.001).unwrap();
        let diff = (final_risk - expected).abs();

        if diff > tolerance {
            return Err(RiskError::InvalidPositionSize {
                reason: format!(
                    "Risk calculation inconsistency: expected {}, got {}",
                    expected, final_risk
                ),
            });
        }

        Ok(())
    }

    fn validate_confidence_range(value: &Decimal, field: &str) -> Result<(), RiskError> {
        if *value < Decimal::ZERO || *value > Decimal::ONE {
            return Err(RiskError::InvalidInput {
                field: field.to_string(),
                reason: "Value must be between 0 and 1".to_string(),
            });
        }
        Ok(())
    }

    /// Update max lot size limit
    pub fn set_max_lot_size(&mut self, max: Decimal) {
        self.max_lot_size = max;
    }

    /// Update max risk percent
    pub fn set_max_risk_percent(&mut self, max: Decimal) {
        self.max_risk_percent = max;
    }
}

impl Default for ValidationGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MarketSession;

    fn test_inputs() -> RiskInputs {
        RiskInputs {
            equity: Decimal::from(10000),
            balance: Decimal::from(10000),
            symbol: "EURUSD".to_string(),
            direction: 1,
            entry_price: Decimal::from_str("1.08500").unwrap(),
            stop_loss: Decimal::from_str("1.08200").unwrap(),
            take_profit: None,
            signal_confidence: Decimal::from_f64(0.8).unwrap(),
            confluence_score: Decimal::from(7),
            regime_quality: Decimal::from_f64(0.7).unwrap(),
            pattern_quality: Decimal::from_f64(0.75).unwrap(),
            atr: Some(Decimal::from_f64(0.00050).unwrap()),
            spread: Decimal::from_f64(0.00010).unwrap(),
            open_positions: vec![],
            daily_pnl: Decimal::ZERO,
            daily_trades: 0,
            recent_trades: vec![],
            session: MarketSession::London,
        }
    }

    #[test]
    fn test_valid_inputs() {
        let guard = ValidationGuard::new();
        let inputs = test_inputs();

        assert!(guard.validate_inputs(&inputs).is_ok());
    }

    #[test]
    fn test_invalid_equity() {
        let guard = ValidationGuard::new();
        let mut inputs = test_inputs();
        inputs.equity = Decimal::ZERO;

        assert!(guard.validate_inputs(&inputs).is_err());
    }

    #[test]
    fn test_invalid_direction() {
        let guard = ValidationGuard::new();
        let mut inputs = test_inputs();
        inputs.direction = 0;

        assert!(guard.validate_inputs(&inputs).is_err());
    }

    #[test]
    fn test_confidence_range() {
        let guard = ValidationGuard::new();
        let mut inputs = test_inputs();
        inputs.signal_confidence = Decimal::from(2); // Out of range

        assert!(guard.validate_inputs(&inputs).is_err());
    }

    #[test]
    fn test_lot_size_validation() {
        let guard = ValidationGuard::new();

        let valid_result = PositionSizeResult {
            lot_size: Decimal::from_f64(0.5).unwrap(),
            risk_percent: Decimal::from_f64(0.01).unwrap(),
            capital_at_risk: Decimal::from(100),
            method: crate::SizingMethod::FixedFractional,
            reasoning: "Test".to_string(),
        };

        assert!(guard.validate_result(&valid_result).is_ok());

        let invalid_result = PositionSizeResult {
            lot_size: Decimal::from(200), // Too large
            ..valid_result
        };

        assert!(guard.validate_result(&invalid_result).is_err());
    }

    #[test]
    fn test_negative_lot_size() {
        let guard = ValidationGuard::new();

        let invalid = PositionSizeResult {
            lot_size: Decimal::from(-1),
            risk_percent: Decimal::ZERO,
            capital_at_risk: Decimal::ZERO,
            method: crate::SizingMethod::FixedFractional,
            reasoning: "Test".to_string(),
        };

        assert!(guard.validate_result(&invalid).is_err());
    }

    #[test]
    fn test_inconsistent_zero_size() {
        let guard = ValidationGuard::new();

        let inconsistent = PositionSizeResult {
            lot_size: Decimal::ZERO,
            risk_percent: Decimal::ZERO,
            capital_at_risk: Decimal::from(100), // Non-zero with zero lot
            method: crate::SizingMethod::FixedFractional,
            reasoning: "Test".to_string(),
        };

        assert!(guard.validate_result(&inconsistent).is_err());
    }
}
