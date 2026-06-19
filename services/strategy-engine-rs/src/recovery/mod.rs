#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryState {
    None,
    Slow,
    Normal,
    Strong,
    Exceptional,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryEngine {
    pub stable_cycles: u64,
}

impl RecoveryEngine {
    pub fn new() -> Self {
        Self { stable_cycles: 0 }
    }

    pub fn record_cycle(&mut self, is_stable: bool) {
        if is_stable {
            self.stable_cycles += 1;
        } else {
            self.stable_cycles = 0; // Reset on instability
        }
    }

    pub fn state(&self) -> RecoveryState {
        if self.stable_cycles >= 20 {
            RecoveryState::Exceptional
        } else if self.stable_cycles >= 10 {
            RecoveryState::Strong
        } else if self.stable_cycles >= 5 {
            RecoveryState::Normal
        } else if self.stable_cycles >= 2 {
            RecoveryState::Slow
        } else {
            RecoveryState::None
        }
    }
}

impl Default for RecoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}
