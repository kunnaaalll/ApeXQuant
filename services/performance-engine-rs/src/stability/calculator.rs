use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use super::models::StabilityMetrics;
use super::states::StabilityState;

pub struct StabilityCalculator;

impl StabilityCalculator {
    pub fn calculate(
        annualized_return: Decimal,
        annualized_volatility: Decimal,
        downside_deviation: Decimal,
        max_drawdown: Decimal,
        ulcer_index: Decimal,
        profitable_periods: u32,
        total_periods: u32,
        variance: Decimal,
    ) -> StabilityMetrics {
        
        let sharpe_ratio = if annualized_volatility.is_zero() {
            if annualized_return > Decimal::ZERO { dec!(100.0) } else { Decimal::ZERO }
        } else {
            (annualized_return / annualized_volatility).min(dec!(100.0)).max(dec!(-100.0))
        };

        let sortino_ratio = if downside_deviation.is_zero() {
            if annualized_return > Decimal::ZERO { dec!(100.0) } else { Decimal::ZERO }
        } else {
            (annualized_return / downside_deviation).min(dec!(100.0)).max(dec!(-100.0))
        };

        let calmar_ratio = if max_drawdown.is_zero() {
            if annualized_return > Decimal::ZERO { dec!(100.0) } else { Decimal::ZERO }
        } else {
            (annualized_return / max_drawdown.abs()).min(dec!(100.0)).max(dec!(-100.0))
        };

        let recovery_factor = if max_drawdown.is_zero() {
            dec!(100.0)
        } else {
            (annualized_return / max_drawdown.abs()).min(dec!(100.0)).max(dec!(-100.0))
        };

        let consistency = if total_periods == 0 {
            Decimal::ZERO
        } else {
            Decimal::from(profitable_periods) / Decimal::from(total_periods)
        };

        let stability_score = ((sharpe_ratio * dec!(10.0)).max(Decimal::ZERO) + (consistency * dec!(100.0))) / dec!(2.0);
        let stability_score = stability_score.min(dec!(100.0)).max(Decimal::ZERO);

        StabilityMetrics {
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            ulcer_index,
            recovery_factor,
            consistency,
            variance,
            stability_score,
        }
    }

    pub fn determine_state(metrics: &StabilityMetrics) -> StabilityState {
        if metrics.stability_score > dec!(80.0) {
            StabilityState::Excellent
        } else if metrics.stability_score > dec!(60.0) {
            StabilityState::Strong
        } else if metrics.stability_score > dec!(40.0) {
            StabilityState::Stable
        } else if metrics.stability_score > dec!(20.0) {
            StabilityState::Weak
        } else {
            StabilityState::Critical
        }
    }
}
