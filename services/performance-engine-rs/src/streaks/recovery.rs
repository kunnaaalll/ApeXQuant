use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone)]
pub struct RecoveryFactor {
    pub recovery_progress: Decimal,
    pub in_recovery: bool,
}

impl RecoveryFactor {
    pub fn calculate(recent_wins_after_drawdown: u32, required_wins: u32) -> Self {
        if required_wins == 0 {
            return Self {
                recovery_progress: dec!(1.0),
                in_recovery: false,
            };
        }

        let mut progress = Decimal::from(recent_wins_after_drawdown) / Decimal::from(required_wins);
        if progress > dec!(1.0) {
            progress = dec!(1.0);
        }

        Self {
            recovery_progress: progress,
            in_recovery: progress < dec!(1.0),
        }
    }
}
