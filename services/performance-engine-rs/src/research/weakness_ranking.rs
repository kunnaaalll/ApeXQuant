use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaknessState {
    Watchlist,
    Weak,
    Danger,
    Forbidden,
}

/// Identifies the worst performers — must be avoided or reduced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaknessRanking {
    pub rank: u32,               // 1 = worst
    pub dimension: String,       // "symbol", "session", "pattern", "regime", "timeframe"
    pub label: String,
    pub expectancy: Decimal,     // negative or near-zero = weak
    pub profit_factor: Decimal,
    pub max_drawdown: Decimal,
    pub confidence: Decimal,
    pub trade_count: u32,
    pub state: WeaknessState,
    pub explanation: String,
}

pub struct WeaknessRankingEngine;

impl WeaknessRankingEngine {
    /// Ranks weaknesses — worst (most negative combined score) at rank 1.
    /// Stable tiebreak: label (lexicographic).
    pub fn rank(mut candidates: Vec<WeaknessRanking>) -> Vec<WeaknessRanking> {
        // Assign states before sorting
        for c in candidates.iter_mut() {
            c.state = Self::classify(c.expectancy, c.profit_factor, c.max_drawdown);
        }

        candidates.sort_by(|a, b| {
            let score_a = a.expectancy * a.profit_factor;
            let score_b = b.expectancy * b.profit_factor;
            // ascending — most negative first
            score_a
                .cmp(&score_b)
                .then(a.label.cmp(&b.label))
        });

        for (i, c) in candidates.iter_mut().enumerate() {
            c.rank = (i + 1) as u32;
        }

        candidates
    }

    fn classify(expectancy: Decimal, profit_factor: Decimal, max_drawdown: Decimal) -> WeaknessState {
        use rust_decimal_macros::dec;
        if expectancy < dec!(-0.10) || profit_factor < dec!(0.70) || max_drawdown > dec!(0.30) {
            WeaknessState::Forbidden
        } else if expectancy < dec!(-0.03) || profit_factor < dec!(0.90) || max_drawdown > dec!(0.20) {
            WeaknessState::Danger
        } else if expectancy < dec!(0.02) || profit_factor < dec!(1.0) {
            WeaknessState::Weak
        } else {
            WeaknessState::Watchlist
        }
    }
}
