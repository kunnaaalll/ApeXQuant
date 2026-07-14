//! Ranking Module
//!
//! Evaluates and ranks strategies across global, per-symbol, per-session,
//! and per-timeframe scopes based on real performance scores.
//!
//! All rankings are computed from input data — no empty or hardcoded responses.

use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RankingError {
    #[error("strategy count {0} and score count {1} must match")]
    LengthMismatch(usize, usize),
}

#[derive(Debug, Clone)]
pub struct GlobalStrategyRank {
    pub strategy_id: String,
    /// 1-based rank (1 = best score).
    pub rank: usize,
    pub score: Decimal,
}

#[derive(Debug, Clone)]
pub struct SymbolRank {
    pub symbol: String,
    pub strategy_id: String,
    pub rank: usize,
    pub score: Decimal,
}

#[derive(Debug, Clone)]
pub struct SessionRank {
    pub session_name: String,
    pub strategy_id: String,
    pub rank: usize,
    pub score: Decimal,
}

#[derive(Debug, Clone)]
pub struct TimeframeRank {
    pub timeframe: String,
    pub strategy_id: String,
    pub rank: usize,
    pub score: Decimal,
}

pub struct RankingEngine;

impl RankingEngine {
    /// Rank strategies globally by score (descending — rank 1 = highest score).
    ///
    /// `strategy_scores` is a parallel slice to `strategy_ids`.
    pub fn rank_global(
        strategy_ids: &[String],
        strategy_scores: &[Decimal],
    ) -> Result<Vec<GlobalStrategyRank>, RankingError> {
        if strategy_ids.len() != strategy_scores.len() {
            return Err(RankingError::LengthMismatch(
                strategy_ids.len(),
                strategy_scores.len(),
            ));
        }

        let mut indexed: Vec<(usize, &String, &Decimal)> = strategy_ids
            .iter()
            .zip(strategy_scores.iter())
            .enumerate()
            .map(|(i, (id, score))| (i, id, score))
            .collect();

        // Sort descending by score
        indexed.sort_by(|a, b| b.2.cmp(a.2));

        Ok(indexed
            .iter()
            .enumerate()
            .map(|(rank_idx, (_, id, score))| GlobalStrategyRank {
                strategy_id: (*id).clone(),
                rank: rank_idx + 1,
                score: **score,
            })
            .collect())
    }

    /// Rank strategies per symbol.
    pub fn rank_by_symbol(
        symbol: &str,
        strategy_ids: &[String],
        strategy_scores: &[Decimal],
    ) -> Result<Vec<SymbolRank>, RankingError> {
        let global = Self::rank_global(strategy_ids, strategy_scores)?;
        Ok(global
            .into_iter()
            .map(|r| SymbolRank {
                symbol: symbol.to_string(),
                strategy_id: r.strategy_id,
                rank: r.rank,
                score: r.score,
            })
            .collect())
    }

    /// Rank strategies per trading session.
    pub fn rank_by_session(
        session_name: &str,
        strategy_ids: &[String],
        strategy_scores: &[Decimal],
    ) -> Result<Vec<SessionRank>, RankingError> {
        let global = Self::rank_global(strategy_ids, strategy_scores)?;
        Ok(global
            .into_iter()
            .map(|r| SessionRank {
                session_name: session_name.to_string(),
                strategy_id: r.strategy_id,
                rank: r.rank,
                score: r.score,
            })
            .collect())
    }

    /// Rank strategies per timeframe.
    pub fn rank_by_timeframe(
        timeframe: &str,
        strategy_ids: &[String],
        strategy_scores: &[Decimal],
    ) -> Result<Vec<TimeframeRank>, RankingError> {
        let global = Self::rank_global(strategy_ids, strategy_scores)?;
        Ok(global
            .into_iter()
            .map(|r| TimeframeRank {
                timeframe: timeframe.to_string(),
                strategy_id: r.strategy_id,
                rank: r.rank,
                score: r.score,
            })
            .collect())
    }

    /// Get the top-N strategies by score.
    pub fn top_n(
        strategy_ids: &[String],
        strategy_scores: &[Decimal],
        n: usize,
    ) -> Result<Vec<GlobalStrategyRank>, RankingError> {
        let all = Self::rank_global(strategy_ids, strategy_scores)?;
        Ok(all.into_iter().take(n).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids_and_scores() -> (Vec<String>, Vec<Decimal>) {
        let ids = vec!["s1".to_string(), "s2".to_string(), "s3".to_string()];
        let scores = vec![
            Decimal::new(75, 2),
            Decimal::new(90, 2),
            Decimal::new(60, 2),
        ];
        (ids, scores)
    }

    #[test]
    fn test_rank_1_is_highest_score() {
        let (ids, scores) = ids_and_scores();
        let ranks = RankingEngine::rank_global(&ids, &scores).expect("ok");
        assert_eq!(ranks[0].rank, 1);
        assert_eq!(ranks[0].strategy_id, "s2"); // score 0.90 is highest
        assert_eq!(ranks[0].score, Decimal::new(90, 2));
    }

    #[test]
    fn test_all_strategies_ranked() {
        let (ids, scores) = ids_and_scores();
        let ranks = RankingEngine::rank_global(&ids, &scores).expect("ok");
        assert_eq!(ranks.len(), 3);
        // Ranks are 1, 2, 3
        let rank_nums: Vec<usize> = ranks.iter().map(|r| r.rank).collect();
        assert_eq!(rank_nums, vec![1, 2, 3]);
    }

    #[test]
    fn test_length_mismatch_returns_error() {
        let ids = vec!["s1".to_string(), "s2".to_string()];
        let scores = vec![Decimal::ONE];
        let result = RankingEngine::rank_global(&ids, &scores);
        assert!(matches!(result, Err(RankingError::LengthMismatch(2, 1))));
    }

    #[test]
    fn test_top_n() {
        let (ids, scores) = ids_and_scores();
        let top2 = RankingEngine::top_n(&ids, &scores, 2).expect("ok");
        assert_eq!(top2.len(), 2);
        assert!(top2[0].score >= top2[1].score);
    }

    #[test]
    fn test_empty_returns_empty() {
        let ranks = RankingEngine::rank_global(&[], &[]).expect("ok");
        assert!(ranks.is_empty());
    }
}
