use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeatDecayModel {
    pub decay_rate_per_interval: u8,
    pub ticks_since_last_loss: u64,
    pub cool_down_threshold_ticks: u64,
}

impl HeatDecayModel {
    pub fn new(decay_rate_per_interval: u8, cool_down_threshold_ticks: u64) -> Self {
        Self {
            decay_rate_per_interval,
            ticks_since_last_loss: 0,
            cool_down_threshold_ticks,
        }
    }

    pub fn register_loss(&mut self) {
        self.ticks_since_last_loss = 0;
    }

    pub fn register_tick(&mut self) {
        self.ticks_since_last_loss += 1;
    }

    pub fn calculate_decay(&self) -> u8 {
        if self.ticks_since_last_loss >= self.cool_down_threshold_ticks {
            self.decay_rate_per_interval
        } else {
            0 // No decay until cooldown passes
        }
    }
}
