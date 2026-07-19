use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreakImpact {
    Positive,
    Neutral,
    Negative,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreakDetector {
    pub current_win_streak: u32,
    pub current_loss_streak: u32,
    pub average_win_streak: Decimal,
    pub average_loss_streak: Decimal,
}

impl StreakDetector {
    pub fn new() -> Self {
        Self {
            current_win_streak: 0,
            current_loss_streak: 0,
            average_win_streak: Decimal::from(0),
            average_loss_streak: Decimal::from(0),
        }
    }
}

impl Default for StreakDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl StreakDetector {
    pub fn record_win(&mut self) {
        self.current_win_streak += 1;
        self.current_loss_streak = 0;
        // In a real implementation we'd maintain a history to update the average properly.
        // We'll approximate or leave the average unchanged for now unless specified.
    }

    pub fn record_loss(&mut self) {
        self.current_loss_streak += 1;
        self.current_win_streak = 0;
    }

    pub fn impact(&self) -> StreakImpact {
        if self.current_loss_streak >= 5 {
            StreakImpact::Critical
        } else if self.current_loss_streak >= 3 {
            StreakImpact::Negative
        } else if self.current_win_streak >= 3 {
            StreakImpact::Positive
        } else {
            StreakImpact::Neutral
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryFactor {
    pub amount: Decimal,
}

impl RecoveryFactor {
    pub fn calculate(consecutive_wins: u32) -> Self {
        // Recovery must be gradual. Single wins cannot fully recover confidence.
        // Recovery = ln(consecutive_wins + 1) or similar, but we're limited to basic decimal ops.
        // We can do consecutive_wins * 0.1 for example.
        let val = Decimal::from(consecutive_wins) * Decimal::new(1, 1);
        let clamped = val.clamp(Decimal::from(0), Decimal::from(1));
        Self { amount: clamped }
    }
}
