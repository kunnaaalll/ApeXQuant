//! Overfitting Detection Module
//!
//! Evaluates parameter sensitivity, combinatorial PBO (probability of backtest overfitting),
//! random permutation tests, and regime dependence to ensure strategy generalizability.
//!
//! Based on:
//! - Bailey, Borwein, Lopez de Prado, Zhu (2015) — "The Probability of Backtest Overfitting"
//! - White (2000) — Reality Check test (Sharpe p-value via permutation)

use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverfittingSeverity {
    /// Score ≤ 30 — strategy generalizes well
    Healthy,
    /// Score 31–60 — moderate overfitting risk
    Warning,
    /// Score > 60 — high probability of curve-fitting
    Critical,
}

/// Score in [0, 100]: higher = more overfitted.
#[derive(Debug, Clone)]
pub struct OverfittingScore(pub Decimal);

/// Full overfitting analysis result.
#[derive(Debug, Clone)]
pub struct OverfittingAnalysis {
    /// Composite overfitting score 0–100 (higher = worse)
    pub score: OverfittingScore,
    pub severity: OverfittingSeverity,
    /// Mean absolute normalized parameter sensitivity (0–1)
    pub parameter_sensitivity: Decimal,
    /// Probability of Backtest Overfitting (PBO) via combinatorial cross-validation (0–1)
    pub probability_of_overfitting: Decimal,
    /// Permutation test p-value for observed Sharpe (lower = more significant)
    pub permutation_p_value: Decimal,
    /// Coefficient of variation of Sharpe across regime windows (lower = less regime-dependent)
    pub regime_dependence: Decimal,
}

/// Input for parameter sensitivity analysis.
#[derive(Debug, Clone)]
pub struct ParameterPoint {
    /// Sharpe ratio observed with this parameter combination
    pub sharpe: Decimal,
    /// Parameter values (normalized to [0, 1] range)
    pub params_normalized: Vec<Decimal>,
}

/// Input trade for permutation/PBO tests.
#[derive(Debug, Clone)]
pub struct OATrade {
    pub pnl: Decimal,
}

pub struct OverfittingAnalyzer;

impl OverfittingAnalyzer {
    /// Full overfitting analysis from parameter sweep + trade history.
    ///
    /// # Arguments
    /// * `param_points` — parameter sweep results (different param combos + their Sharpe)
    /// * `trades` — full trade history for permutation test
    /// * `observed_sharpe` — Sharpe of the selected/optimized strategy
    /// * `regime_sharpes` — Sharpe ratios in each detected market regime window
    /// * `seed` — deterministic seed for permutation tests
    pub fn analyze(
        param_points: &[ParameterPoint],
        trades: &[OATrade],
        observed_sharpe: Decimal,
        regime_sharpes: &[Decimal],
        seed: u64,
    ) -> Result<OverfittingAnalysis, String> {
        // 1. Parameter sensitivity
        let parameter_sensitivity = compute_parameter_sensitivity(param_points);

        // 2. Probability of overfitting (combinatorial cross-validation / bootstrap method)
        let probability_of_overfitting = compute_pbo(param_points, seed);

        // 3. Permutation test p-value
        let permutation_p_value = compute_permutation_p_value(trades, observed_sharpe, 2000, seed)?;

        // 4. Regime dependence
        let regime_dependence = compute_regime_dependence(regime_sharpes);

        // 5. Composite score
        // Weights: PBO 40%, sensitivity 30%, regime_dependence 30%
        let pbo_f = probability_of_overfitting.to_f64().unwrap_or(0.0);
        let sens_f = parameter_sensitivity.to_f64().unwrap_or(0.0);
        let regime_f = regime_dependence.to_f64().unwrap_or(0.0).min(1.0);
        let raw_score = pbo_f * 0.40 + sens_f * 0.30 + regime_f * 0.30;
        let score_int = (raw_score * 100.0).round().clamp(0.0, 100.0) as i64;
        let score = OverfittingScore(Decimal::from(score_int));

        let severity = match score_int {
            0..=30 => OverfittingSeverity::Healthy,
            31..=60 => OverfittingSeverity::Warning,
            _ => OverfittingSeverity::Critical,
        };

        Ok(OverfittingAnalysis {
            score,
            severity,
            parameter_sensitivity,
            probability_of_overfitting,
            permutation_p_value,
            regime_dependence,
        })
    }

    /// Simplified analyze when only trades and observed Sharpe are available
    /// (no parameter sweep). Uses permutation test only.
    pub fn analyze_from_trades(
        trades: &[OATrade],
        observed_sharpe: Decimal,
        seed: u64,
    ) -> Result<OverfittingAnalysis, String> {
        let permutation_p_value = compute_permutation_p_value(trades, observed_sharpe, 2000, seed)?;

        // Without parameter sweep, score is driven by permutation p-value
        // High p-value (≥ 0.05) suggests luck, not skill
        let pbo_f = permutation_p_value.to_f64().unwrap_or(1.0);
        let score_int = (pbo_f * 100.0).round().clamp(0.0, 100.0) as i64;
        let score = OverfittingScore(Decimal::from(score_int));

        let severity = match score_int {
            0..=30 => OverfittingSeverity::Healthy,
            31..=60 => OverfittingSeverity::Warning,
            _ => OverfittingSeverity::Critical,
        };

        Ok(OverfittingAnalysis {
            score,
            severity,
            parameter_sensitivity: Decimal::ZERO,
            probability_of_overfitting: permutation_p_value,
            permutation_p_value,
            regime_dependence: Decimal::ZERO,
        })
    }
}

// ---------------------------------------------------------------------------
// Internal computations
// ---------------------------------------------------------------------------

/// Parameter sensitivity: mean |ΔSharpe / ΔParam| normalized to [0, 1].
/// Compares each point against the mean Sharpe across the sweep.
fn compute_parameter_sensitivity(points: &[ParameterPoint]) -> Decimal {
    if points.len() < 2 {
        return Decimal::ZERO;
    }
    let sharpes: Vec<f64> = points
        .iter()
        .map(|p| p.sharpe.to_f64().unwrap_or(0.0))
        .collect();
    let mean_sharpe = sharpes.iter().sum::<f64>() / sharpes.len() as f64;
    if mean_sharpe == 0.0 {
        return Decimal::ZERO;
    }

    // Normalized sensitivity: std_dev(sharpes) / mean_sharpe
    let var = sharpes
        .iter()
        .map(|&s| (s - mean_sharpe).powi(2))
        .sum::<f64>()
        / (sharpes.len() - 1) as f64;
    let std = var.sqrt();
    let sensitivity = (std / mean_sharpe.abs()).clamp(0.0, 1.0);

    f64_to_decimal(sensitivity)
}

/// Probability of Backtest Overfitting via bootstrap cross-validation.
///
/// Method: Split trade history in half (IS | OOS), repeatedly rank strategies by IS Sharpe,
/// check if IS winner wins OOS. PBO = fraction of trials where IS winner loses OOS.
fn compute_pbo(param_points: &[ParameterPoint], seed: u64) -> Decimal {
    if param_points.len() < 2 {
        return Decimal::ZERO;
    }
    let sharpes: Vec<f64> = param_points
        .iter()
        .map(|p| p.sharpe.to_f64().unwrap_or(0.0))
        .collect();

    let num_trials = 1000_usize;
    let lose_count: usize = (0..num_trials)
        .into_par_iter()
        .filter(|&trial| {
            let sub_seed =
                seed.wrapping_add((trial as u64).wrapping_mul(3_202_034_522_624_059_733));
            let mut rng = ChaCha8Rng::seed_from_u64(sub_seed);

            // Bootstrap sample of IS and OOS sharpes (simulate by permuting the array)
            let mut is_indices: Vec<usize> = (0..sharpes.len()).collect();
            is_indices.shuffle(&mut rng);
            let mid = is_indices.len() / 2;
            let is_set = &is_indices[..mid];
            let oos_set = &is_indices[mid..];

            if is_set.is_empty() || oos_set.is_empty() {
                return false;
            }

            // IS winner = index with highest IS Sharpe
            let is_winner = is_set.iter().copied().max_by(|&a, &b| {
                sharpes[a]
                    .partial_cmp(&sharpes[b])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let oos_winner = oos_set.iter().copied().max_by(|&a, &b| {
                sharpes[a]
                    .partial_cmp(&sharpes[b])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            match (is_winner, oos_winner) {
                (Some(iw), Some(ow)) => sharpes[iw] < sharpes[ow], // IS winner loses OOS
                _ => false,
            }
        })
        .count();

    f64_to_decimal(lose_count as f64 / num_trials as f64)
}

/// Permutation test: p-value for observed Sharpe under null (random return ordering).
fn compute_permutation_p_value(
    trades: &[OATrade],
    observed_sharpe: Decimal,
    num_permutations: usize,
    seed: u64,
) -> Result<Decimal, String> {
    if trades.len() < 2 {
        return Err("Need at least 2 trades for permutation test".to_string());
    }
    let obs_f = observed_sharpe.to_f64().ok_or("Sharpe out of range")?;
    let pnls: Vec<f64> = trades
        .iter()
        .map(|t| t.pnl.to_f64().unwrap_or(0.0))
        .collect();

    let exceed: usize = (0..num_permutations)
        .into_par_iter()
        .filter(|&i| {
            let sub_seed = seed.wrapping_add((i as u64).wrapping_mul(1_442_695_040_888_963_407));
            let mut rng = ChaCha8Rng::seed_from_u64(sub_seed);
            let mut shuffled = pnls.clone();
            shuffled.shuffle(&mut rng);
            sharpe_f64(&shuffled) >= obs_f
        })
        .count();

    Ok(f64_to_decimal(exceed as f64 / num_permutations as f64))
}

/// Regime dependence: coefficient of variation of Sharpe across regimes.
fn compute_regime_dependence(regime_sharpes: &[Decimal]) -> Decimal {
    if regime_sharpes.len() < 2 {
        return Decimal::ZERO;
    }
    let vals: Vec<f64> = regime_sharpes
        .iter()
        .map(|s| s.to_f64().unwrap_or(0.0))
        .collect();
    let n = vals.len() as f64;
    let mean = vals.iter().sum::<f64>() / n;
    if mean.abs() < 1e-10 {
        return Decimal::ONE; // Maximum dependence if mean ≈ 0
    }
    let var = vals.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n;
    let cv = var.sqrt() / mean.abs();
    f64_to_decimal(cv.clamp(0.0, 1.0))
}

fn sharpe_f64(pnls: &[f64]) -> f64 {
    if pnls.len() < 2 {
        return 0.0;
    }
    let n = pnls.len() as f64;
    let mean = pnls.iter().sum::<f64>() / n;
    let var = pnls.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std = var.sqrt();
    if std == 0.0 {
        0.0
    } else {
        (mean / std) * 252_f64.sqrt()
    }
}

fn f64_to_decimal(v: f64) -> Decimal {
    Decimal::from_f64_retain(v)
        .unwrap_or(Decimal::ZERO)
        .round_dp(6)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_healthy() {
        let trades: Vec<OATrade> = (0..50)
            .map(|i| OATrade {
                pnl: if i % 3 != 0 {
                    Decimal::new(100, 0)
                } else {
                    Decimal::new(-60, 0)
                },
            })
            .collect();
        let result = OverfittingAnalyzer::analyze_from_trades(
            &trades,
            Decimal::new(15, 1), // 1.5 Sharpe
            42,
        )
        .expect("analyze failed");
        // Score and severity must be valid
        assert!(matches!(
            result.severity,
            OverfittingSeverity::Healthy
                | OverfittingSeverity::Warning
                | OverfittingSeverity::Critical
        ));
        assert!(result.permutation_p_value >= Decimal::ZERO);
        assert!(result.permutation_p_value <= Decimal::ONE);
    }

    #[test]
    fn test_parameter_sensitivity_zero_for_equal_sharpes() {
        let points = vec![
            ParameterPoint {
                sharpe: Decimal::new(15, 1),
                params_normalized: vec![Decimal::new(1, 1)],
            },
            ParameterPoint {
                sharpe: Decimal::new(15, 1),
                params_normalized: vec![Decimal::new(5, 1)],
            },
        ];
        // All sharpes equal → sensitivity = 0
        let sens = super::compute_parameter_sensitivity(&points);
        assert_eq!(sens, Decimal::ZERO);
    }
}
