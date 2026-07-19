use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RankTier {
    Forbidden,
    Weak,
    Normal,
    Strong,
    Elite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrategyRank {
    pub score: Decimal,
    pub tier: RankTier,
}

impl StrategyRank {
    pub fn calculate(
        edge: Decimal,
        confidence: Decimal,
        stability: Decimal,
        drawdown: Decimal,
    ) -> Self {
        // edge × confidence × stability ÷ drawdown
        let denominator = if drawdown == Decimal::from(0) {
            Decimal::from(1)
        } else {
            drawdown
        };
        let mut score = (edge * confidence * stability) / denominator;

        if score < Decimal::from(0) {
            score = Decimal::from(0);
        }

        let tier = if score >= Decimal::from(1000) {
            RankTier::Elite
        } else if score >= Decimal::from(500) {
            RankTier::Strong
        } else if score >= Decimal::from(100) {
            RankTier::Normal
        } else if score >= Decimal::from(10) {
            RankTier::Weak
        } else {
            RankTier::Forbidden
        };

        Self { score, tier }
    }
}

pub mod context;
pub use context::*;
