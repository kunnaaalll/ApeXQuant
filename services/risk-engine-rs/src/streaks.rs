use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreakTracker {
    pub current_win_streak: u32,
    pub current_loss_streak: u32,
    pub max_win_streak: u32,
    pub max_loss_streak: u32,
}

impl StreakTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_outcome(&mut self, won: bool) {
        if won {
            self.current_win_streak += 1;
            self.current_loss_streak = 0;
            if self.current_win_streak > self.max_win_streak {
                self.max_win_streak = self.current_win_streak;
            }
        } else {
            self.current_loss_streak += 1;
            self.current_win_streak = 0;
            if self.current_loss_streak > self.max_loss_streak {
                self.max_loss_streak = self.current_loss_streak;
            }
        }
    }
}
