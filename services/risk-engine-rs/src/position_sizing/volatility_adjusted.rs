use rust_decimal::prelude::FromPrimitive;
use crate::{PositionSizeResult, RiskInputs, VolatilityMetrics};
use rust_decimal::Decimal;
use std::str::FromStr;

/// Volatility-adjusted position sizing
///
/// Reduces position size in high volatility, increases in low volatility
/// proportional to account for changing market conditions.
#[derive(Debug, Clone, Copy, Default)]
pub struct VolatilityAdjustedSizing;

impl VolatilityAdjustedSizing {
    /// Adjust position size based on volatility metrics
    pub fn adjust(
        &self,
        current: PositionSizeResult,
        vol_metrics: &VolatilityMetrics,
        inputs: &RiskInputs,
    ) -> PositionSizeResult {
        // Skip if no ATR data available
        let atr = match inputs.atr {
            Some(atr) if atr > Decimal::ZERO => atr,
            _ => return current,
        };

        // Calculate ATR as percentage of price
        let atr_pct = atr / inputs.entry_price;

        // Scale based on volatility regime
        let volatility_multiplier = match vol_metrics.regime {
            crate::volatility::VolatilityRegime::VeryLow => Decimal::from_f64(1.3).unwrap_or(Decimal::ONE),
            crate::volatility::VolatilityRegime::Low => Decimal::from_f64(1.15).unwrap_or(Decimal::ONE),
            crate::volatility::VolatilityRegime::Normal => Decimal::ONE,
            crate::volatility::VolatilityRegime::High => Decimal::from_f64(0.75).unwrap_or(Decimal::ONE),
            crate::volatility::VolatilityRegime::VeryHigh => Decimal::from_f64(0.5).unwrap_or(Decimal::ONE),
            crate::volatility::VolatilityRegime::Extreme => Decimal::from_f64(0.25).unwrap_or(Decimal::ONE),
        };

        // Additional spread impact calculation
        let spread_pct = inputs.spread / inputs.entry_price;
        let spread_impact = if spread_pct > Decimal::from_str("0.0002").unwrap_or(Decimal::ZERO) {
            // Wide spreads reduce position size
            Decimal::ONE - (spread_pct * Decimal::from(1000)) // 2 pip spread = 20% reduction
        } else {
            Decimal::ONE
        };

        let combined_multiplier = volatility_multiplier * spread_impact.max(Decimal::from_f64(0.5).unwrap());

        let new_lot_size = current.lot_size * combined_multiplier;
        let new_capital_at_risk = current.capital_at_risk * combined_multiplier;
        let new_risk_percent = current.risk_percent * combined_multiplier;

        PositionSizeResult {
            lot_size: new_lot_size,
            risk_percent: new_risk_percent,
            capital_at_risk: new_capital_at_risk,
            method: super::SizingMethod::VolatilityAdjusted,
            reasoning: format!(
                "{} Volatility adjustment: {} regime ({}/100 ATR), spread impact: {}/100",
                current.reasoning,
                serde_json::to_string(&vol_metrics.regime).unwrap_or_default(),
                atr_pct * Decimal::from(10000), // Convert to pips
                spread_impact * Decimal::from(100)
            ),
        }
    }

    /// Calculate volatility-based maximum position size
    pub fn max_position_for_volatility(
        &self,
        equity: Decimal,
        atr: Decimal,
        max_risk_multiple: Decimal,
    ) -> Decimal {
        // Maximum position based on ATR - don't risk more than
        // max_risk_multiple * ATR per trade
        equity * max_risk_multiple / atr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::volatility::{VolatilityMetrics, VolatilityRegime};

    fn base_result() -> PositionSizeResult {
        PositionSizeResult {
            lot_size: Decimal::ONE,
            risk_percent: Decimal::from_str("0.01").unwrap(),
            capital_at_risk: Decimal::from(100),
            method: super::super::SizingMethod::FixedFractional,
            reasoning: "Base calculation".to_string(),
        }
    }

    fn test_inputs(atr: Option<Decimal>) -> RiskInputs {
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
            atr,
            spread: Decimal::from_f64(0.0001).unwrap(),
            open_positions: Vec::new(),
            daily_pnl: Decimal::ZERO,
            daily_trades: 0,
            recent_trades: Vec::new(),
            session: crate::MarketSession::London,
        }
    }

    fn vol_metrics(regime: VolatilityRegime) -> VolatilityMetrics {
        VolatilityMetrics {
            regime,
            atr: Decimal::from_str("0.00050").unwrap(),
            relative_volatility: Decimal::ONE,
            spread_expansion: Decimal::ONE,
            timestamp: time::OffsetDateTime::now_utc(),
        }
    }

    #[test]
    fn test_high_volatility_reduction() {
        let sizer = VolatilityAdjustedSizing;
        let base = base_result();
        let inputs = test_inputs(Some(Decimal::from_str("0.00100").unwrap()));
        let metrics = vol_metrics(VolatilityRegime::High);

        let result = sizer.adjust(base, &metrics, &inputs);

        // High volatility should reduce size
        assert!(result.lot_size < Decimal::ONE);
        assert_eq!(result.method, super::super::SizingMethod::VolatilityAdjusted);
    }

    #[test]
    fn test_low_volatility_increase() {
        let sizer = VolatilityAdjustedSizing;
        let base = base_result();
        let inputs = test_inputs(Some(Decimal::from_str("0.00020").unwrap()));
        let metrics = vol_metrics(VolatilityRegime::Low);

        let result = sizer.adjust(base, &metrics, &inputs);

        // Low volatility should increase size
        assert!(result.lot_size > Decimal::ONE);
    }

    #[test]
    fn test_no_atr_fallback() {
        let sizer = VolatilityAdjustedSizing;
        let base = base_result();
        let inputs = test_inputs(None); // No ATR
        let metrics = vol_metrics(VolatilityRegime::Normal);

        let result = sizer.adjust(base.clone(), &metrics, &inputs);

        // Should return unchanged
        assert_eq!(result.lot_size, base.lot_size);
    }
}
