use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct VolatilityAdjustedSizer {
    pub base_risk_fraction: Decimal, // e.g. 0.02
    pub target_volatility: Decimal,  // target ATR or std dev
}

impl VolatilityAdjustedSizer {
    pub fn new(base_risk_fraction: Decimal, target_volatility: Decimal) -> Self {
        Self {
            base_risk_fraction,
            target_volatility,
        }
    }

    /// Calculate size: (equity * base_risk_fraction * (target_volatility / current_volatility)) / stop_loss_distance
    pub fn calculate_size(
        &self,
        equity: Decimal,
        stop_loss_distance: Decimal,
        current_volatility: Decimal,
    ) -> Decimal {
        if stop_loss_distance <= Decimal::ZERO || current_volatility <= Decimal::ZERO || equity <= Decimal::ZERO {
            return Decimal::ZERO;
        }

        let ratio = self.target_volatility / current_volatility;
        // Limit multiplier to avoid extreme size expansion in ultra low volatility
        let multiplier = ratio.min(dec!(2.0));
        let adjusted_risk = self.base_risk_fraction * multiplier;
        let risk_amount = equity * adjusted_risk;

        (risk_amount / stop_loss_distance).trunc_with_scale(4)
    }
}
