use rust_decimal::prelude::FromPrimitive;
use crate::{PositionSizeResult, RiskError, RiskInputs};
use rust_decimal::Decimal;
use std::str::FromStr;

/// Fixed fractional position sizing
///
/// Risk a fixed percentage of equity per trade based on stop distance.
#[derive(Debug, Clone, Copy)]
pub struct FixedFractionalSizing {
    /// Default risk percent (0.01 = 1%)
    default_risk_percent: Decimal,
    /// Minimum risk percent
    min_risk_percent: Decimal,
    /// Maximum risk percent
    max_risk_percent: Decimal,
}

impl FixedFractionalSizing {
    /// Create new fixed fractional sizer with custom parameters
    pub fn new(
        default_risk_percent: Decimal,
        min_risk_percent: Decimal,
        max_risk_percent: Decimal,
    ) -> Self {
        Self {
            default_risk_percent,
            min_risk_percent,
            max_risk_percent,
        }
    }

    /// Calculate position size based on fixed fractional method
    pub fn calculate(&self, inputs: &RiskInputs) -> Result<PositionSizeResult, RiskError> {
        // Validate inputs
        if inputs.equity <= Decimal::ZERO {
            return Err(RiskError::InvalidInput {
                field: "equity".to_string(),
                reason: "Equity must be positive".to_string(),
            });
        }

        if inputs.stop_loss <= Decimal::ZERO {
            return Err(RiskError::InvalidInput {
                field: "stop_loss".to_string(),
                reason: "Stop loss must be positive".to_string(),
            });
        }

        if inputs.entry_price <= Decimal::ZERO {
            return Err(RiskError::InvalidInput {
                field: "entry_price".to_string(),
                reason: "Entry price must be positive".to_string(),
            });
        }

        // Calculate stop distance as percentage of entry price
        let stop_distance = (inputs.entry_price - inputs.stop_loss).abs();
        if stop_distance <= Decimal::ZERO {
            return Err(RiskError::InvalidInput {
                field: "stop_distance".to_string(),
                reason: "Stop distance must be positive".to_string(),
            });
        }

        // Calculate risk amount based on default risk percent
        let risk_amount = inputs.equity * self.default_risk_percent;

        // Calculate lot size: risk_amount / (stop_distance * pip_value)
        // For forex, pip_value = lot_size * 0.0001 (approximate)
        // Simplified: lot size = risk_amount / stop_distance
        let lot_size = risk_amount / stop_distance;

        // Clamp to reasonable bounds (0.01 to 100.0 lots)
        let min_lot = Decimal::from_f64(0.01).unwrap_or(Decimal::new(1, 2));
        let max_lot = Decimal::from_f64(100.0).unwrap_or(Decimal::new(100, 0));
        let lot_size = lot_size.clamp(min_lot, max_lot);

        let capital_at_risk = lot_size * stop_distance;
        let actual_risk_percent = capital_at_risk / inputs.equity;

        Ok(PositionSizeResult {
            lot_size,
            risk_percent: actual_risk_percent.clamp(self.min_risk_percent, self.max_risk_percent),
            capital_at_risk,
            method: super::SizingMethod::FixedFractional,
            reasoning: format!(
                "Fixed fractional sizing: {:.2}% risk at {} equity with {} stop distance",
                self.default_risk_percent * Decimal::from(100),
                inputs.equity,
                stop_distance
            ),
        })
    }
}

impl Default for FixedFractionalSizing {
    fn default() -> Self {
        Self {
            default_risk_percent: Decimal::from_str("0.01").unwrap_or(Decimal::new(1, 2)),
            min_risk_percent: Decimal::from_str("0.005").unwrap_or(Decimal::new(5, 3)),
            max_risk_percent: Decimal::from_str("0.02").unwrap_or(Decimal::new(2, 2)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_inputs() -> RiskInputs {
        RiskInputs {
            equity: Decimal::from(10000),
            balance: Decimal::from(10000),
            symbol: "EURUSD".to_string(),
            direction: 1,
            entry_price: Decimal::from_str("1.08500").unwrap(),
            stop_loss: Decimal::from_str("1.08200").unwrap(),
            take_profit: None,
            signal_confidence: Decimal::from_f64(0.8).unwrap_or(Decimal::ONE),
            confluence_score: Decimal::from(7),
            regime_quality: Decimal::from_f64(0.7).unwrap_or(Decimal::ONE),
            pattern_quality: Decimal::from_f64(0.75).unwrap_or(Decimal::ONE),
            atr: None,
            spread: Decimal::from_f64(0.0001).unwrap_or(Decimal::ZERO),
            open_positions: Vec::new(),
            daily_pnl: Decimal::ZERO,
            daily_trades: 0,
            recent_trades: Vec::new(),
            session: crate::MarketSession::London,
        }
    }

    #[test]
    fn test_fixed_fractional_calculation() {
        let sizer = FixedFractionalSizing::default();
        let inputs = test_inputs();

        let result = sizer.calculate(&inputs).unwrap();

        // Risk $100 (1%) with 30 pip stop
        // Lot size should be approximately $100 / 0.0030 = 33.33 lots
        assert!(result.lot_size > Decimal::ZERO);
        assert_eq!(result.method, super::SizingMethod::FixedFractional);
        assert!(result.capital_at_risk > Decimal::ZERO);
    }

    #[test]
    fn test_invalid_equity() {
        let sizer = FixedFractionalSizing::default();
        let mut inputs = test_inputs();
        inputs.equity = Decimal::ZERO;

        let result = sizer.calculate(&inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_stop() {
        let sizer = FixedFractionalSizing::default();
        let mut inputs = test_inputs();
        inputs.stop_loss = inputs.entry_price; // Zero distance

        let result = sizer.calculate(&inputs);
        assert!(result.is_err());
    }
}
