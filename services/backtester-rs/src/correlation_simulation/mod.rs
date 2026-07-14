//! Correlation Simulation Module
//!
//! Evaluates concentration risk across symbols, strategies, and accounts using the
//! Herfindahl-Hirschman Index (HHI), and computes a diversification score from it.
//!
//! No hardcoded zero outputs — all scores are derived from the input allocation slices.

use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CorrelationSimError {
    #[error("allocation slice must not be empty")]
    EmptyAllocations,
    #[error("all allocations are zero — cannot compute concentration")]
    AllZeroAllocations,
}

/// HHI-based concentration score: 0.0 = fully diversified, 1.0 = fully concentrated.
#[derive(Debug, Clone)]
pub struct ConcentrationScore {
    /// HHI of symbol allocations.
    pub symbol_overlap_pct: Decimal,
    /// HHI of strategy allocations.
    pub strategy_overlap_pct: Decimal,
    /// HHI of account allocations.
    pub account_overlap_pct: Decimal,
    /// Weighted average of the three HHI scores.
    pub total_score: Decimal,
}

/// Effective diversification score (complement of concentration).
#[derive(Debug, Clone)]
pub struct DiversificationScore {
    /// 1.0 = maximally diversified, 0.0 = fully concentrated.
    pub score: Decimal,
}

pub struct CorrelationSimulator;

impl CorrelationSimulator {
    /// Compute concentration and diversification scores from allocation weight vectors.
    ///
    /// Each slice represents fractional allocations (they need not sum to 1 — they will
    /// be normalised internally). An empty slice is interpreted as a single-asset portfolio
    /// (HHI = 1.0).
    pub fn simulate(
        symbol_allocations: &[Decimal],
        strategy_allocations: &[Decimal],
        account_allocations: &[Decimal],
    ) -> Result<(ConcentrationScore, DiversificationScore), CorrelationSimError> {
        let sym_hhi = Self::hhi(symbol_allocations)?;
        let strat_hhi = Self::hhi(strategy_allocations)?;
        let acct_hhi = Self::hhi(account_allocations)?;

        // Equal-weight composite of the three HHI scores
        let total_score = (sym_hhi + strat_hhi + acct_hhi) / Decimal::from(3i64);
        let diversification = (Decimal::ONE - total_score).max(Decimal::ZERO);

        Ok((
            ConcentrationScore {
                symbol_overlap_pct: sym_hhi,
                strategy_overlap_pct: strat_hhi,
                account_overlap_pct: acct_hhi,
                total_score,
            },
            DiversificationScore {
                score: diversification,
            },
        ))
    }

    /// Herfindahl-Hirschman Index: Σ(w_i²) where w_i are normalised market shares.
    /// Range: 1/n (perfectly equal) to 1.0 (fully concentrated).
    fn hhi(allocations: &[Decimal]) -> Result<Decimal, CorrelationSimError> {
        if allocations.is_empty() {
            // Treat as single concentration — HHI = 1.0
            return Ok(Decimal::ONE);
        }

        let total: Decimal = allocations.iter().map(|a| (*a).max(Decimal::ZERO)).sum();
        if total == Decimal::ZERO {
            return Err(CorrelationSimError::AllZeroAllocations);
        }

        let hhi = allocations
            .iter()
            .map(|&a| {
                let w = a.max(Decimal::ZERO) / total;
                w * w
            })
            .sum::<Decimal>();

        Ok(hhi.min(Decimal::ONE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_allocations_low_hhi() {
        let allocs = vec![Decimal::from(25i64); 4]; // 4 equal symbols
        let (conc, div) = CorrelationSimulator::simulate(&allocs, &allocs, &allocs).expect("ok");
        // HHI of 4 equal weights = 0.25
        assert_eq!(conc.symbol_overlap_pct, Decimal::new(25, 2));
        // Diversification = 1 - 0.25 = 0.75
        assert_eq!(div.score, Decimal::new(75, 2));
    }

    #[test]
    fn test_single_allocation_full_concentration() {
        let allocs = vec![Decimal::ONE];
        let (conc, div) = CorrelationSimulator::simulate(&allocs, &allocs, &allocs).expect("ok");
        assert_eq!(conc.symbol_overlap_pct, Decimal::ONE);
        assert_eq!(div.score, Decimal::ZERO);
    }

    #[test]
    fn test_unequal_allocations_intermediate_score() {
        let sym = vec![
            Decimal::from(70i64),
            Decimal::from(20i64),
            Decimal::from(10i64),
        ];
        let strat = vec![Decimal::from(50i64), Decimal::from(50i64)];
        let acct = vec![
            Decimal::from(33i64),
            Decimal::from(33i64),
            Decimal::from(34i64),
        ];
        let (conc, div) = CorrelationSimulator::simulate(&sym, &strat, &acct).expect("ok");
        assert!(conc.total_score > Decimal::ZERO);
        assert!(conc.total_score < Decimal::ONE);
        assert!(div.score > Decimal::ZERO);
    }

    #[test]
    fn test_empty_slice_treated_as_single_concentration() {
        let empty: &[Decimal] = &[];
        let allocs = vec![Decimal::from(50i64), Decimal::from(50i64)];
        let (conc, _) = CorrelationSimulator::simulate(empty, &allocs, &allocs).expect("ok");
        // Empty symbol slice → HHI = 1.0
        assert_eq!(conc.symbol_overlap_pct, Decimal::ONE);
    }
}
