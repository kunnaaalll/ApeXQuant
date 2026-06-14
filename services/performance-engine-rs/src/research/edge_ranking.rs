use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A ranked edge — the best historical configurations found during research.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRanking {
    pub rank: u32,
    pub strategy_id: String,
    pub dimension: String,
    pub label: String,
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub sharpe_approx: Decimal,
    pub confidence: Decimal,
    pub trade_count: u32,
    pub explanation: String,
}

pub struct EdgeRankingEngine;

impl EdgeRankingEngine {
    /// Ranks edge candidates by combined score: expectancy × profit_factor × confidence.
    /// Stable tiebreak: strategy_id then label (lexicographic).
    pub fn rank(mut candidates: Vec<EdgeRanking>) -> Vec<EdgeRanking> {
        candidates.sort_by(|a, b| {
            let score_a = a.expectancy * a.profit_factor * a.confidence;
            let score_b = b.expectancy * b.profit_factor * b.confidence;
            score_b
                .cmp(&score_a)
                .then(a.strategy_id.cmp(&b.strategy_id))
                .then(a.label.cmp(&b.label))
        });

        for (i, c) in candidates.iter_mut().enumerate() {
            c.rank = (i + 1) as u32;
        }

        candidates
    }
}
