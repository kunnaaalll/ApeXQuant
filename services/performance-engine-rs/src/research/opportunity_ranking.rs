use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A ranked candidate for institutional research — symbol, session, pattern, regime, or timeframe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityRanking {
    pub rank: u32,
    pub dimension: String,   // e.g. "symbol", "session", "pattern", "regime", "timeframe"
    pub label: String,        // e.g. "EURUSD", "London", "BreakoutPullback"
    pub edge: Decimal,        // Signed expectancy
    pub confidence: Decimal,  // [0, 1]
    pub sample_quality: Decimal, // [0, 1] — penalised for small N
    pub historical_evidence: u32, // trade count backing the ranking
    pub explanation: String,
}

pub struct OpportunityRankingEngine;

impl OpportunityRankingEngine {
    /// Ranks candidates by a composite opportunity score: edge × confidence × sample_quality.
    /// Deterministic: equal scores are broken by label (lexicographic) to guarantee stable ordering.
    pub fn rank(mut candidates: Vec<OpportunityRanking>) -> Vec<OpportunityRanking> {
        candidates.sort_by(|a, b| {
            let score_a = a.edge * a.confidence * a.sample_quality;
            let score_b = b.edge * b.confidence * b.sample_quality;
            score_b
                .cmp(&score_a)
                .then(a.label.cmp(&b.label)) // deterministic tiebreak
        });

        for (i, c) in candidates.iter_mut().enumerate() {
            c.rank = (i + 1) as u32;
        }

        candidates
    }
}
