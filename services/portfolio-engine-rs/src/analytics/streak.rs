// src/analytics/streak.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct StreakAnalytics {
    pub current_winning_streak: u64,
    pub current_losing_streak: u64,
    pub max_winning_streak: u64,
    pub max_losing_streak: u64,
    
    /// Speed at which the portfolio recovers from a losing streak
    pub recovery_speed: rust_decimal::Decimal,
    
    /// Measured impact of streaks on the overall expectancy
    pub streak_impact: rust_decimal::Decimal,
}

impl StreakAnalytics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_win(&mut self) {
        self.current_losing_streak = 0;
        self.current_winning_streak += 1;
        if self.current_winning_streak > self.max_winning_streak {
            self.max_winning_streak = self.current_winning_streak;
        }
    }

    pub fn record_loss(&mut self) {
        self.current_winning_streak = 0;
        self.current_losing_streak += 1;
        if self.current_losing_streak > self.max_losing_streak {
            self.max_losing_streak = self.current_losing_streak;
        }
    }
}
