use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternState {
    Emerging,
    Maturing,
    Strong,
    Weakening,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: Uuid,
    pub state: PatternState,
    pub win_rate: Decimal,
    pub observation_count: u64,
}

pub struct PatternTracker {
    pub patterns: std::collections::HashMap<Uuid, Pattern>,
}

impl Default for PatternTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternTracker {
    pub fn new() -> Self {
        Self {
            patterns: std::collections::HashMap::new(),
        }
    }

    pub fn update_pattern(&mut self, pattern_id: Uuid, win: bool) {
        let pattern = self.patterns.entry(pattern_id).or_insert(Pattern {
            id: pattern_id,
            state: PatternState::Emerging,
            win_rate: Decimal::new(0, 0),
            observation_count: 0,
        });
        
        let mut wins = pattern.win_rate * Decimal::from(pattern.observation_count);
        if win {
            wins += Decimal::ONE;
        }
        pattern.observation_count += 1;
        pattern.win_rate = wins / Decimal::from(pattern.observation_count);
        
        // Simple state machine logic
        if pattern.observation_count > 100 {
            if pattern.win_rate > Decimal::new(60, 2) {
                pattern.state = PatternState::Strong;
            } else if pattern.win_rate < Decimal::new(40, 2) {
                pattern.state = PatternState::Weakening;
            } else {
                pattern.state = PatternState::Maturing;
            }
        }
        
        if pattern.state == PatternState::Weakening && pattern.win_rate < Decimal::new(30, 2) && pattern.observation_count > 200 {
            pattern.state = PatternState::Dead;
        }
    }
}
