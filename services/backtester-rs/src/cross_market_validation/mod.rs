//! Cross-Market Validation Module
//!
//! Evaluates strategy generalisation across different market classes using real
//! per-market performance data and applies overfit penalties based on the
//! proportion of markets tested vs markets that passed validation.

use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum CrossMarketError {
    #[error("no market performance data provided")]
    NoMarketData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketClass {
    Forex,
    Indices,
    Metals,
    Crypto,
    Futures,
}

impl MarketClass {
    pub fn label(&self) -> &'static str {
        match self {
            MarketClass::Forex => "Forex",
            MarketClass::Indices => "Indices",
            MarketClass::Metals => "Metals",
            MarketClass::Crypto => "Crypto",
            MarketClass::Futures => "Futures",
        }
    }
}

/// Per-market performance statistics for cross-market validation.
#[derive(Debug, Clone)]
pub struct MarketPerformance {
    pub market: MarketClass,
    pub trade_count: usize,
    pub win_rate: Decimal,
    /// Expectancy in currency units per trade.
    pub expectancy: Decimal,
    pub profit_factor: Decimal,
    pub max_drawdown: Decimal,
}

/// Validation result for a single market.
#[derive(Debug, Clone)]
pub struct CrossMarketResult {
    pub strategy_id: Uuid,
    pub market: MarketClass,
    /// Raw performance score [0.0, 1.0].
    pub score: Decimal,
    /// Penalty applied to the score based on overfit risk.
    pub overfit_penalty: Decimal,
    /// Final adjusted score = score - overfit_penalty.
    pub adjusted_score: Decimal,
    /// Whether the strategy is valid on this market after penalty.
    pub passes: bool,
}

/// Aggregated cross-market validation result.
#[derive(Debug, Clone)]
pub struct CrossMarketValidationResult {
    pub strategy_id: Uuid,
    pub market_results: Vec<CrossMarketResult>,
    /// Fraction of tested markets on which the strategy passes.
    pub generalisation_score: Decimal,
    /// Whether the strategy generalises well enough (≥ 60% market pass rate).
    pub is_generalisable: bool,
}

pub struct StandardValidator;

impl StandardValidator {
    /// Minimum score to consider a strategy valid on a market.
    fn pass_threshold() -> Decimal {
        Decimal::new(35, 2)
    }

    /// Validate a strategy across multiple markets.
    ///
    /// The overfit penalty increases with the number of markets tested (more fitting
    /// opportunities = higher overfit risk) and is reduced by the pass fraction
    /// (if the strategy passes most markets, overfit is less likely).
    pub fn validate(
        strategy_id: Uuid,
        market_performances: &[MarketPerformance],
    ) -> Result<CrossMarketValidationResult, CrossMarketError> {
        if market_performances.is_empty() {
            return Err(CrossMarketError::NoMarketData);
        }

        let n = market_performances.len();

        // Compute raw scores first
        let raw_scores: Vec<Decimal> = market_performances
            .iter()
            .map(Self::compute_raw_score)
            .collect();

        // Pass count before penalty
        let passes_before_penalty = raw_scores
            .iter()
            .filter(|&&s| s >= Self::pass_threshold())
            .count();
        let pass_fraction = if n > 0 {
            Decimal::from(passes_before_penalty as i64) / Decimal::from(n as i64)
        } else {
            Decimal::ZERO
        };

        // Overfit penalty: higher when many markets tested and low pass rate.
        // penalty = (1 - pass_fraction) * (n_markets - 1) / (n_markets * 10)
        let overfit_penalty = if n > 1 {
            (Decimal::ONE - pass_fraction) * Decimal::from((n - 1) as i64)
                / (Decimal::from(n as i64) * Decimal::from(10i64))
        } else {
            Decimal::ZERO
        };

        let mut market_results: Vec<CrossMarketResult> = Vec::with_capacity(n);
        let mut final_passes = 0usize;

        for (mp, &raw_score) in market_performances.iter().zip(raw_scores.iter()) {
            let adjusted_score = (raw_score - overfit_penalty).max(Decimal::ZERO);
            let passes = adjusted_score >= Self::pass_threshold();
            if passes {
                final_passes += 1;
            }

            market_results.push(CrossMarketResult {
                strategy_id,
                market: mp.market,
                score: raw_score,
                overfit_penalty,
                adjusted_score,
                passes,
            });
        }

        let generalisation_score = Decimal::from(final_passes as i64) / Decimal::from(n as i64);
        let is_generalisable = generalisation_score >= Decimal::new(60, 2);

        Ok(CrossMarketValidationResult {
            strategy_id,
            market_results,
            generalisation_score,
            is_generalisable,
        })
    }

    /// Compute a raw performance score [0.0, 1.0] for a single market.
    ///
    /// Composite of: win_rate (40%), positive expectancy (30%), profit factor (20%),
    /// and drawdown control (10%).
    fn compute_raw_score(mp: &MarketPerformance) -> Decimal {
        if mp.trade_count == 0 {
            return Decimal::ZERO;
        }
        let win_score = mp.win_rate.max(Decimal::ZERO).min(Decimal::ONE) * Decimal::new(40, 2);
        let exp_score = if mp.expectancy > Decimal::ZERO {
            (mp.expectancy / Decimal::from(10i64)).min(Decimal::ONE) * Decimal::new(30, 2)
        } else {
            Decimal::ZERO
        };
        let pf_score = if mp.profit_factor > Decimal::ONE {
            ((mp.profit_factor - Decimal::ONE) / Decimal::from(2i64)).min(Decimal::ONE)
                * Decimal::new(20, 2)
        } else {
            Decimal::ZERO
        };
        let dd_score = {
            let dd_cap = Decimal::new(20, 2);
            let dd = mp.max_drawdown.min(dd_cap).max(Decimal::ZERO);
            (Decimal::ONE - dd / dd_cap) * Decimal::new(10, 2)
        };
        (win_score + exp_score + pf_score + dd_score).min(Decimal::ONE)
    }
}

// ── Trait for extensibility ──────────────────────────────────────────────────

pub trait CrossMarketValidator {
    fn validate_strategy(
        &self,
        strategy_id: Uuid,
        market_performances: &[MarketPerformance],
    ) -> Result<CrossMarketValidationResult, CrossMarketError>;
}

impl CrossMarketValidator for StandardValidator {
    fn validate_strategy(
        &self,
        strategy_id: Uuid,
        market_performances: &[MarketPerformance],
    ) -> Result<CrossMarketValidationResult, CrossMarketError> {
        Self::validate(strategy_id, market_performances)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strong_market(market: MarketClass) -> MarketPerformance {
        MarketPerformance {
            market,
            trade_count: 100,
            win_rate: Decimal::new(60, 2),
            expectancy: Decimal::from(12i64),
            profit_factor: Decimal::new(18, 1),
            max_drawdown: Decimal::new(8, 2),
        }
    }

    fn weak_market(market: MarketClass) -> MarketPerformance {
        MarketPerformance {
            market,
            trade_count: 15,
            win_rate: Decimal::new(35, 2),
            expectancy: Decimal::ZERO,
            profit_factor: Decimal::new(8, 1),
            max_drawdown: Decimal::new(18, 2),
        }
    }

    #[test]
    fn test_empty_returns_error() {
        let result = StandardValidator::validate(Uuid::new_v4(), &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_strong_market_passes() {
        let perfs = vec![strong_market(MarketClass::Forex)];
        let result = StandardValidator::validate(Uuid::new_v4(), &perfs).expect("ok");
        assert!(result.market_results[0].passes);
    }

    #[test]
    fn test_weak_market_fails() {
        let perfs = vec![weak_market(MarketClass::Crypto)];
        let result = StandardValidator::validate(Uuid::new_v4(), &perfs).expect("ok");
        assert!(!result.market_results[0].passes);
    }

    #[test]
    fn test_generalisation_score_computed() {
        let perfs = vec![
            strong_market(MarketClass::Forex),
            strong_market(MarketClass::Indices),
            strong_market(MarketClass::Metals),
        ];
        let result = StandardValidator::validate(Uuid::new_v4(), &perfs).expect("ok");
        assert!(result.generalisation_score > Decimal::new(5, 1));
        assert!(result.is_generalisable);
    }
}
