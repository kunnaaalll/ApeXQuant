use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextRankTier {
    Forbidden,
    Weak,
    Normal,
    Strong,
    Elite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextRank {
    pub score: Decimal,
    pub tier: ContextRankTier,
}

impl ContextRank {
    pub fn calculate(
        expectancy: Decimal,
        confidence: Decimal,
        stability: Decimal,
        drawdown: Decimal,
    ) -> Self {
        // expectancy × confidence × stability ÷ drawdown
        let denominator = if drawdown == Decimal::from(0) {
            Decimal::from(1)
        } else {
            drawdown
        };
        let mut score = (expectancy * confidence * stability) / denominator;

        if score < Decimal::from(0) {
            score = Decimal::from(0);
        }

        let tier = if score >= Decimal::from(1000) {
            ContextRankTier::Elite
        } else if score >= Decimal::from(500) {
            ContextRankTier::Strong
        } else if score >= Decimal::from(100) {
            ContextRankTier::Normal
        } else if score >= Decimal::from(10) {
            ContextRankTier::Weak
        } else {
            ContextRankTier::Forbidden
        };

        Self { score, tier }
    }
}
