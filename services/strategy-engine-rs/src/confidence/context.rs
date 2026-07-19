use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextConfidenceTier {
    VeryLow,
    Low,
    Normal,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextConfidenceScore {
    score: Decimal,
}

impl ContextConfidenceScore {
    pub fn new(score: Decimal) -> Self {
        let clamped = score.clamp(Decimal::from(0), Decimal::from(100));
        Self { score: clamped }
    }

    pub fn value(&self) -> Decimal {
        self.score
    }

    pub fn calculate(
        sample_quality: Decimal,
        stability: Decimal,
        degradation: Decimal,
        drawdown: Decimal,
    ) -> Self {
        let mut base = (sample_quality + stability) / Decimal::from(2);

        let penalty = (drawdown * Decimal::from(2)) + degradation;
        base -= penalty;

        Self::new(base)
    }

    pub fn tier(&self) -> ContextConfidenceTier {
        if self.score >= Decimal::from(80) {
            ContextConfidenceTier::VeryHigh
        } else if self.score >= Decimal::from(60) {
            ContextConfidenceTier::High
        } else if self.score >= Decimal::from(40) {
            ContextConfidenceTier::Normal
        } else if self.score >= Decimal::from(20) {
            ContextConfidenceTier::Low
        } else {
            ContextConfidenceTier::VeryLow
        }
    }
}
