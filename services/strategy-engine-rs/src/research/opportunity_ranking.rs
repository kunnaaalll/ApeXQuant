use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpportunityRanking {
    pub score: Decimal,
}

impl OpportunityRanking {
    pub fn calculate(edge: Decimal, confidence: Decimal, sample_quality: Decimal) -> Self {
        Self {
            score: edge * confidence * sample_quality,
        }
    }
}
