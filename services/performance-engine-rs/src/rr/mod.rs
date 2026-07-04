use rust_decimal::Decimal;

pub struct RiskRewardAnalyzer;

impl RiskRewardAnalyzer {
    pub fn calculate_rr(
        average_win: Decimal,
        average_loss: Decimal,
    ) -> Decimal {
        if average_loss == Decimal::ZERO {
            Decimal::ZERO
        } else {
            average_win / average_loss.abs()
        }
    }
}
