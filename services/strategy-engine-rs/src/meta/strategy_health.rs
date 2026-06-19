use rust_decimal::Decimal;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrategyHealth {
    score: Decimal,
}

impl StrategyHealth {
    pub fn new(initial_score: u32) -> Self {
        let mut score = Decimal::from(initial_score);
        if score > Decimal::from(100) {
            score = Decimal::from(100);
        }
        Self { score }
    }

    pub fn score(&self) -> Decimal {
        self.score
    }

    pub fn collapse(&mut self, new_score: u32) {
        let new_decimal = Decimal::from(new_score);
        if new_decimal < self.score {
            self.score = new_decimal;
        }
    }

    pub fn recover(&mut self, recovery_amount: Decimal) {
        let max_recovery = Decimal::from(5);
        let actual_recovery = if recovery_amount > max_recovery {
            max_recovery
        } else {
            recovery_amount
        };

        if actual_recovery > Decimal::from(0) {
            self.score += actual_recovery;
            if self.score > Decimal::from(100) {
                self.score = Decimal::from(100);
            }
        }
    }

    pub fn deteriorate(&mut self, damage: Decimal) {
        if damage > Decimal::from(0) {
            self.score -= damage;
            if self.score < Decimal::from(0) {
                self.score = Decimal::from(0);
            }
        }
    }
}

impl Default for StrategyHealth {
    fn default() -> Self {
        Self::new(100)
    }
}

impl PartialOrd for StrategyHealth {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StrategyHealth {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}
