use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreakState {
    Positive,
    Neutral,
    Negative,
    Critical,
}

#[derive(Debug, Clone)]
pub struct StreakDetector {
    pub current_streak: i32,
    pub max_win_streak: i32,
    pub max_loss_streak: i32,
    pub average_win_streak: Decimal,
    pub average_loss_streak: Decimal,
    pub state: StreakState,
}

impl StreakDetector {
    pub fn evaluate(
        current_streak: i32,
        max_win_streak: i32,
        max_loss_streak: i32,
        average_win_streak: Decimal,
        average_loss_streak: Decimal,
    ) -> Self {
        let state = if current_streak <= -5 || current_streak <= -max_loss_streak {
            StreakState::Critical
        } else if current_streak < -2 {
            StreakState::Negative
        } else if current_streak > 2 {
            StreakState::Positive
        } else {
            StreakState::Neutral
        };

        Self {
            current_streak,
            max_win_streak,
            max_loss_streak,
            average_win_streak,
            average_loss_streak,
            state,
        }
    }
}
