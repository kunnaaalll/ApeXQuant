use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub struct EdgeRankingEngine;

impl EdgeRankingEngine {
    /// Rank edge: score = win_rate * profit_factor
    pub fn rank_edge(win_rate: Decimal, profit_factor: Decimal) -> Decimal {
        if win_rate <= Decimal::ZERO || profit_factor <= Decimal::ZERO {
            return Decimal::ZERO;
        }

        win_rate * profit_factor
    }
}
