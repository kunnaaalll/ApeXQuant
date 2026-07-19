use rust_decimal::Decimal;

pub mod context;
pub mod penalty;

pub use context::*;
pub use penalty::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfidenceTier {
    VeryLow,
    Low,
    Normal,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfidenceScore {
    score: Decimal,
}

impl ConfidenceScore {
    pub fn new(score: Decimal) -> Self {
        let clamped = score.clamp(Decimal::from(0), Decimal::from(100));
        Self { score: clamped }
    }

    pub fn value(&self) -> Decimal {
        self.score
    }

    pub fn calculate(
        sample_quality: Decimal,
        edge_score: Decimal,
        stability: Decimal,
        penalty: ConfidencePenalty,
    ) -> Self {
        let base = (sample_quality + edge_score + stability) / Decimal::from(3);
        let adjusted = base - penalty.amount;

        Self::new(adjusted)
    }

    pub fn tier(&self) -> ConfidenceTier {
        if self.score >= Decimal::from(80) {
            ConfidenceTier::VeryHigh
        } else if self.score >= Decimal::from(60) {
            ConfidenceTier::High
        } else if self.score >= Decimal::from(40) {
            ConfidenceTier::Normal
        } else if self.score >= Decimal::from(20) {
            ConfidenceTier::Low
        } else {
            ConfidenceTier::VeryLow
        }
    }
}
