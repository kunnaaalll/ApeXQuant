
#[derive(Debug, Clone)]
pub struct CooldownModel {
    pub required_cooldown_ticks: u64,
    pub current_cooldown_ticks: u64,
}

impl CooldownModel {
    pub fn new(required_cooldown_ticks: u64) -> Self {
        Self {
            required_cooldown_ticks,
            current_cooldown_ticks: 0,
        }
    }

    pub fn tick(&mut self, is_loss: bool) {
        if is_loss {
            // Losses reset progress immediately
            self.current_cooldown_ticks = 0;
        } else {
            if self.current_cooldown_ticks < self.required_cooldown_ticks {
                self.current_cooldown_ticks += 1;
            }
        }
    }

    pub fn is_cooldown_complete(&self) -> bool {
        self.current_cooldown_ticks >= self.required_cooldown_ticks
    }

    pub fn force_reset(&mut self) {
        self.current_cooldown_ticks = 0;
    }
}
