use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone)]
pub struct ConfidenceScoreCalculator;

impl ConfidenceScoreCalculator {
    /// Compute confidence score based on performance metrics.
    /// Returns score in [0.0, 1.0].
    pub fn calculate(win_rate: Decimal, profit_factor: Decimal, sharpe_ratio: Decimal) -> Decimal {
        if win_rate <= Decimal::ZERO || profit_factor <= Decimal::ZERO {
            return Decimal::ZERO;
        }

        // Weighted confidence:
        // 40% win rate, 40% profit factor, 20% Sharpe ratio (clamped to 3.0 max)
        let w_win = win_rate * dec!(0.4);

        let pf_score = (profit_factor / dec!(3.0)).min(dec!(1.0)) * dec!(0.4);

        let sharpe_score = (sharpe_ratio.max(Decimal::ZERO) / dec!(3.0)).min(dec!(1.0)) * dec!(0.2);

        let total = w_win + pf_score + sharpe_score;
        total.min(dec!(1.0)).max(dec!(0.0))
    }
}
