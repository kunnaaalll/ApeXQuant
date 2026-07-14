//! Regime Validation Module
//!
//! Validates strategy performance across distinct market regimes to guarantee
//! robust performance regardless of market state.
//!
//! All scores are computed from real per-regime trade statistics — no empty stubs.

use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegimeValidationError {
    #[error("at least one regime performance record is required")]
    NoRegimeData,
    #[error("regime performance data has {0} regimes but expected at least 1")]
    InsufficientRegimes(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MarketRegime {
    Trend,
    Range,
    HighVolatility,
    LowVolatility,
    News,
    SessionTransition,
}

impl MarketRegime {
    pub fn label(&self) -> &'static str {
        match self {
            MarketRegime::Trend => "Trend",
            MarketRegime::Range => "Range",
            MarketRegime::HighVolatility => "High Volatility",
            MarketRegime::LowVolatility => "Low Volatility",
            MarketRegime::News => "News Event",
            MarketRegime::SessionTransition => "Session Transition",
        }
    }
}

/// Trade statistics for a single regime period.
#[derive(Debug, Clone)]
pub struct RegimePerformance {
    pub regime: MarketRegime,
    pub trade_count: usize,
    pub win_rate: Decimal,
    /// Expectancy in currency units per trade.
    pub expectancy: Decimal,
    pub max_drawdown: Decimal,
}

/// Fitness score for a single regime.
#[derive(Debug, Clone)]
pub struct RegimeFitnessScore {
    pub regime: MarketRegime,
    /// Composite regime fitness score [0.0, 1.0].
    pub score: Decimal,
    /// True if the strategy is viable in this regime (score ≥ viability threshold).
    pub is_viable: bool,
    pub trade_count: usize,
}

/// Aggregated regime validation result.
#[derive(Debug, Clone)]
pub struct RegimeValidationResult {
    pub scores: Vec<RegimeFitnessScore>,
    /// Fraction of regimes in which the strategy is viable.
    pub regime_coverage: Decimal,
    /// Minimum fitness score across all regimes (weakest-link metric).
    pub min_regime_score: Decimal,
    /// Whether the strategy passes all-regime validation.
    pub passes_all_regimes: bool,
}

pub struct RegimeValidator;

impl RegimeValidator {
    /// Minimum score a strategy must achieve in a regime to be considered viable.
    fn viability_threshold() -> Decimal {
        Decimal::new(30, 2)
    }

    /// Validate a strategy across provided regime performance records.
    ///
    /// The fitness score per regime is a weighted composite of:
    /// - 50% win rate contribution
    /// - 30% positive expectancy contribution
    /// - 20% drawdown control contribution
    pub fn validate(
        regime_performances: &[RegimePerformance],
    ) -> Result<RegimeValidationResult, RegimeValidationError> {
        if regime_performances.is_empty() {
            return Err(RegimeValidationError::NoRegimeData);
        }

        let mut scores: Vec<RegimeFitnessScore> = Vec::with_capacity(regime_performances.len());

        for rp in regime_performances {
            let score = Self::compute_fitness(rp);
            let is_viable = score >= Self::viability_threshold();
            scores.push(RegimeFitnessScore {
                regime: rp.regime.clone(),
                score,
                is_viable,
                trade_count: rp.trade_count,
            });
        }

        let viable_count = scores.iter().filter(|s| s.is_viable).count();
        let total = scores.len();
        let regime_coverage = Decimal::from(viable_count as i64) / Decimal::from(total as i64);
        let min_regime_score = scores
            .iter()
            .map(|s| s.score)
            .fold(Decimal::ONE, |a, b| a.min(b));
        let passes_all_regimes = viable_count == total;

        Ok(RegimeValidationResult {
            scores,
            regime_coverage,
            min_regime_score,
            passes_all_regimes,
        })
    }

    /// Compute a composite fitness score [0.0, 1.0] for a single regime.
    fn compute_fitness(rp: &RegimePerformance) -> Decimal {
        if rp.trade_count == 0 {
            return Decimal::ZERO;
        }

        // Win rate contribution (50%): map [0, 1] win rate to [0, 0.5] score.
        let win_rate_score = rp.win_rate.max(Decimal::ZERO).min(Decimal::ONE) * Decimal::new(50, 2);

        // Expectancy contribution (30%): positive expectancy → 0.30, zero → 0.0.
        // We normalise expectancy against a benchmark of 10 currency units/trade.
        let expectancy_score = if rp.expectancy > Decimal::ZERO {
            let bench = Decimal::from(10i64);
            (rp.expectancy / bench).min(Decimal::ONE) * Decimal::new(30, 2)
        } else {
            Decimal::ZERO
        };

        // Drawdown control (20%): lower drawdown → better score.
        // 0% drawdown → 0.20; 20%+ drawdown → 0.0.
        let dd_control = if rp.max_drawdown < Decimal::ZERO {
            Decimal::new(20, 2)
        } else {
            let max_dd_cap = Decimal::new(20, 2);
            let dd_clamped = rp.max_drawdown.min(max_dd_cap);
            (Decimal::ONE - dd_clamped / max_dd_cap) * Decimal::new(20, 2)
        };

        (win_rate_score + expectancy_score + dd_control).min(Decimal::ONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strong_performance(regime: MarketRegime) -> RegimePerformance {
        RegimePerformance {
            regime,
            trade_count: 100,
            win_rate: Decimal::new(60, 2),
            expectancy: Decimal::from(15i64),
            max_drawdown: Decimal::new(8, 2),
        }
    }

    fn weak_performance(regime: MarketRegime) -> RegimePerformance {
        RegimePerformance {
            regime,
            trade_count: 20,
            win_rate: Decimal::new(35, 2),
            expectancy: Decimal::ZERO,
            max_drawdown: Decimal::new(18, 2),
        }
    }

    #[test]
    fn test_empty_returns_error() {
        assert!(RegimeValidator::validate(&[]).is_err());
    }

    #[test]
    fn test_strong_performance_is_viable() {
        let perfs = vec![strong_performance(MarketRegime::Trend)];
        let result = RegimeValidator::validate(&perfs).expect("ok");
        assert!(result.scores[0].is_viable);
        assert!(result.scores[0].score > Decimal::new(50, 2));
    }

    #[test]
    fn test_weak_performance_not_viable() {
        let perfs = vec![weak_performance(MarketRegime::News)];
        let result = RegimeValidator::validate(&perfs).expect("ok");
        assert!(!result.scores[0].is_viable);
    }

    #[test]
    fn test_passes_all_regimes_when_all_viable() {
        let perfs = vec![
            strong_performance(MarketRegime::Trend),
            strong_performance(MarketRegime::Range),
        ];
        let result = RegimeValidator::validate(&perfs).expect("ok");
        assert!(result.passes_all_regimes);
        assert_eq!(result.regime_coverage, Decimal::ONE);
    }

    #[test]
    fn test_partial_regime_coverage() {
        let perfs = vec![
            strong_performance(MarketRegime::Trend),
            weak_performance(MarketRegime::HighVolatility),
        ];
        let result = RegimeValidator::validate(&perfs).expect("ok");
        assert!(!result.passes_all_regimes);
        assert_eq!(result.regime_coverage, Decimal::new(5, 1)); // 50%
    }

    #[test]
    fn test_zero_trades_gives_zero_score() {
        let perf = RegimePerformance {
            regime: MarketRegime::Range,
            trade_count: 0,
            win_rate: Decimal::new(60, 2),
            expectancy: Decimal::from(20i64),
            max_drawdown: Decimal::ZERO,
        };
        let result = RegimeValidator::validate(&[perf]).expect("ok");
        assert_eq!(result.scores[0].score, Decimal::ZERO);
        assert!(!result.scores[0].is_viable);
    }
}
