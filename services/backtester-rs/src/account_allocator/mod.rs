//! Account Allocator Module
//!
//! Determines how capital is allocated across different strategies and accounts
//! based on five production-grade allocation models.
//!
//! All models are computed from real input data — no hardcoded zeros.

use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AllocatorError {
    #[error("no accounts to allocate to")]
    NoAccounts,
    #[error("total capital must be positive, got {0}")]
    NonPositiveCapital(Decimal),
    #[error("scores/volatility slice length {0} must match account count {1}")]
    LengthMismatch(usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocationModel {
    /// Equal capital to each account.
    EqualWeight,
    /// Inverse-volatility weighting (lower vol → larger allocation).
    VolatilityWeight,
    /// Risk-parity: allocate so each account contributes equally to total portfolio risk.
    RiskWeight,
    /// Weight by each account's confidence score (normalised 0-1).
    ConfidenceWeight,
    /// Weight by rolling Sharpe ratio (negative Sharpe → zero allocation).
    PerformanceWeight,
}

#[derive(Debug, Clone)]
pub struct AllocationRecommendation {
    pub account_id: String,
    pub allocated_capital: Decimal,
    pub weight: Decimal,
}

/// Capital utilisation efficiency: ratio of deployed capital to available capital,
/// adjusted by allocation entropy (Gini coefficient of weight distribution).
#[derive(Debug, Clone)]
pub struct CapitalEfficiencyScore {
    /// 0.0 (fully concentrated) to 1.0 (perfectly diversified / efficient).
    pub score: Decimal,
}

pub struct Allocator;

impl Allocator {
    /// Allocate capital across accounts using the specified model.
    ///
    /// `weights_or_scores` is model-specific:
    /// - `EqualWeight`: ignored (may be empty).
    /// - `VolatilityWeight`: per-account annualised volatility (σ).
    /// - `RiskWeight`: per-account risk units (e.g. VaR or σ × exposure).
    /// - `ConfidenceWeight`: per-account confidence score [0, 1].
    /// - `PerformanceWeight`: per-account rolling Sharpe ratio.
    pub fn allocate(
        model: &AllocationModel,
        total_capital: Decimal,
        account_ids: &[String],
        weights_or_scores: &[Decimal],
    ) -> Result<(Vec<AllocationRecommendation>, CapitalEfficiencyScore), AllocatorError> {
        if account_ids.is_empty() {
            return Err(AllocatorError::NoAccounts);
        }
        if total_capital <= Decimal::ZERO {
            return Err(AllocatorError::NonPositiveCapital(total_capital));
        }

        let n = account_ids.len();

        // For non-equal-weight models, scores slice must match account count.
        if *model != AllocationModel::EqualWeight && weights_or_scores.len() != n {
            return Err(AllocatorError::LengthMismatch(weights_or_scores.len(), n));
        }

        let weights = match model {
            AllocationModel::EqualWeight => {
                vec![Decimal::ONE; n]
            }
            AllocationModel::VolatilityWeight => {
                // Inverse volatility: w_i = (1/σ_i) / Σ(1/σ_j)
                let inv: Vec<Decimal> = weights_or_scores
                    .iter()
                    .map(|&v| {
                        if v > Decimal::ZERO {
                            Decimal::ONE / v
                        } else {
                            Decimal::ZERO
                        }
                    })
                    .collect();
                inv
            }
            AllocationModel::RiskWeight => {
                // Risk parity: w_i = (1/risk_i) / Σ(1/risk_j)  — same as inv-vol in principle
                let inv: Vec<Decimal> = weights_or_scores
                    .iter()
                    .map(|&v| {
                        if v > Decimal::ZERO {
                            Decimal::ONE / v
                        } else {
                            Decimal::ZERO
                        }
                    })
                    .collect();
                inv
            }
            AllocationModel::ConfidenceWeight => {
                // Direct confidence scores; clip to [0, 1]
                weights_or_scores
                    .iter()
                    .map(|&s| s.max(Decimal::ZERO).min(Decimal::ONE))
                    .collect()
            }
            AllocationModel::PerformanceWeight => {
                // Sharpe-based: clip negative Sharpe to zero
                weights_or_scores
                    .iter()
                    .map(|&s| s.max(Decimal::ZERO))
                    .collect()
            }
        };

        // Normalise weights to sum to 1.0
        let total_weight: Decimal = weights.iter().sum();
        let normalised: Vec<Decimal> = if total_weight > Decimal::ZERO {
            weights.iter().map(|w| w / total_weight).collect()
        } else {
            // Fallback: equal weight when all inputs are zero
            vec![Decimal::ONE / Decimal::from(n as i64); n]
        };

        let recommendations: Vec<AllocationRecommendation> = account_ids
            .iter()
            .zip(normalised.iter())
            .map(|(id, &w)| AllocationRecommendation {
                account_id: id.clone(),
                allocated_capital: total_capital * w,
                weight: w,
            })
            .collect();

        // Capital Efficiency Score = 1 - Gini(weights)
        // Gini = 0 → perfectly equal → fully efficient; Gini = 1 → fully concentrated.
        let score = Decimal::ONE - Self::gini(&normalised);

        Ok((recommendations, CapitalEfficiencyScore { score }))
    }

    /// Gini coefficient of a weight distribution (0 = perfect equality, 1 = total inequality).
    fn gini(weights: &[Decimal]) -> Decimal {
        let n = weights.len();
        if n <= 1 {
            return Decimal::ZERO;
        }
        let mut sorted = weights.to_vec();
        sorted.sort();
        let mut numerator = Decimal::ZERO;
        for (i, &w) in sorted.iter().enumerate() {
            numerator += Decimal::from((2 * i + 1) as i64 - n as i64) * w;
        }
        let mean = weights.iter().sum::<Decimal>() / Decimal::from(n as i64);
        if mean == Decimal::ZERO {
            return Decimal::ZERO;
        }
        (numerator / (Decimal::from(n as i64) * mean)).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("acc_{}", i)).collect()
    }

    #[test]
    fn test_equal_weight_splits_evenly() {
        let (recs, eff) = Allocator::allocate(
            &AllocationModel::EqualWeight,
            Decimal::from(10_000i64),
            &ids(4),
            &[],
        )
        .expect("ok");
        for r in &recs {
            assert_eq!(r.allocated_capital, Decimal::from(2500i64));
            assert_eq!(r.weight, Decimal::new(25, 2));
        }
        assert!(eff.score > Decimal::ZERO);
    }

    #[test]
    fn test_allocations_sum_to_total_capital() {
        let capital = Decimal::from(100_000i64);
        let scores = vec![Decimal::new(5, 1), Decimal::new(8, 1), Decimal::new(3, 1)];
        let (recs, _) = Allocator::allocate(
            &AllocationModel::ConfidenceWeight,
            capital,
            &ids(3),
            &scores,
        )
        .expect("ok");
        let sum: Decimal = recs.iter().map(|r| r.allocated_capital).sum();
        // Allow for small rounding error
        let diff = (sum - capital).abs();
        assert!(diff < Decimal::new(1, 2), "sum {sum} ≠ capital {capital}");
    }

    #[test]
    fn test_volatility_weight_higher_vol_gets_less() {
        let capital = Decimal::from(10_000i64);
        let vols = vec![Decimal::new(1, 1), Decimal::new(5, 1)]; // 0.1 vs 0.5
        let (recs, _) =
            Allocator::allocate(&AllocationModel::VolatilityWeight, capital, &ids(2), &vols)
                .expect("ok");
        // Account with lower vol (idx 0) gets more capital
        assert!(recs[0].allocated_capital > recs[1].allocated_capital);
    }

    #[test]
    fn test_no_accounts_returns_error() {
        let result = Allocator::allocate(
            &AllocationModel::EqualWeight,
            Decimal::from(1000i64),
            &[],
            &[],
        );
        assert!(matches!(result, Err(AllocatorError::NoAccounts)));
    }

    #[test]
    fn test_negative_capital_returns_error() {
        let result = Allocator::allocate(
            &AllocationModel::EqualWeight,
            Decimal::from(-1i64),
            &ids(2),
            &[],
        );
        assert!(matches!(result, Err(AllocatorError::NonPositiveCapital(_))));
    }
}
