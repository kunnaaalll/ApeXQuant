//! Portfolio Optimizer — Mean-Variance Optimization
//!
//! Implements:
//! - Minimum Variance Portfolio (analytical Lagrangian solution)
//! - Maximum Sharpe Portfolio (golden-section search on efficient frontier)
//! - Risk Budget Portfolio (equal marginal contribution to risk)
//! - Volatility-Targeted Portfolio (scale weights to hit target vol)
//!
//! Uses `nalgebra` for linear algebra (f64) internally. All inputs/outputs are
//! `rust_decimal::Decimal` — f64 is only used for Cholesky/matrix inversion.

use nalgebra::{DMatrix, DVector};
use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConstraints {
    pub max_weight_per_asset: Decimal,
    pub min_weight_per_asset: Decimal,
    pub target_volatility: Option<Decimal>,
    pub risk_free_rate: Decimal,
}

impl Default for OptimizationConstraints {
    fn default() -> Self {
        Self {
            max_weight_per_asset: Decimal::new(40, 2),  // 40%
            min_weight_per_asset: Decimal::new(5, 2),   // 5%
            target_volatility: None,
            risk_free_rate: Decimal::ZERO,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedPortfolio {
    pub weights: Vec<(String, Decimal)>,
    pub expected_return: Decimal,
    pub estimated_volatility: Decimal,
    pub sharpe_ratio: Decimal,
    pub optimization_method: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationMethod {
    MinimumVariance,
    MaximumSharpe,
    RiskBudget,
    VolatilityTargeted,
    EqualWeight,
}

pub struct PortfolioOptimizer;

impl Default for PortfolioOptimizer {
    fn default() -> Self { Self::new() }
}

impl PortfolioOptimizer {
    pub fn new() -> Self { Self }

    /// Optimize portfolio using specified method.
    ///
    /// # Arguments
    /// * `assets` — asset identifiers
    /// * `expected_returns` — μ vector (annualised, same order as assets)
    /// * `covariance_matrix` — Σ matrix (n×n, annualised, row-major)
    /// * `constraints` — weight bounds and optional target volatility
    /// * `method` — optimization objective
    pub fn optimize(
        &self,
        assets: &[String],
        expected_returns: &[Decimal],
        covariance_matrix: &[Vec<Decimal>],
        constraints: &OptimizationConstraints,
        method: OptimizationMethod,
    ) -> Result<OptimizedPortfolio, String> {
        let n = assets.len();
        if n == 0 {
            return Err("Asset list is empty".to_string());
        }
        if expected_returns.len() != n {
            return Err(format!("expected_returns length {} != assets length {}", expected_returns.len(), n));
        }
        if covariance_matrix.len() != n || covariance_matrix.iter().any(|row| row.len() != n) {
            return Err("Covariance matrix dimensions do not match asset count".to_string());
        }

        // Convert to f64 for nalgebra
        let mu = dec_vec_to_f64(expected_returns)?;
        let sigma = dec_matrix_to_f64(covariance_matrix)?;
        let min_w = constraints.min_weight_per_asset.to_f64()
            .ok_or("min_weight out of range")?;
        let max_w = constraints.max_weight_per_asset.to_f64()
            .ok_or("max_weight out of range")?;
        let rf = constraints.risk_free_rate.to_f64()
            .ok_or("risk_free_rate out of range")?;

        let raw_weights = match method {
            OptimizationMethod::MinimumVariance => {
                minimum_variance(&sigma, n)?
            }
            OptimizationMethod::MaximumSharpe => {
                maximum_sharpe(&mu, &sigma, n, rf)?
            }
            OptimizationMethod::RiskBudget => {
                risk_budget_equal(&sigma, n)?
            }
            OptimizationMethod::VolatilityTargeted => {
                let target = constraints.target_volatility
                    .and_then(|v| v.to_f64())
                    .unwrap_or(0.15); // default 15% annualised
                volatility_targeted(&mu, &sigma, n, rf, target)?
            }
            OptimizationMethod::EqualWeight => {
                vec![1.0 / n as f64; n]
            }
        };

        // Clip to [min_w, max_w] and renormalise
        let weights = clip_and_normalise(raw_weights, min_w, max_w, n);

        // Compute portfolio statistics
        let wvec = DVector::from_vec(weights.clone());
        let mu_vec = DVector::from_vec(mu);
        let sigma_mat = DMatrix::from_row_slice(n, n, &sigma);

        let port_return = wvec.dot(&mu_vec);
        let variance = (wvec.transpose() * &sigma_mat * &wvec)[(0, 0)];
        let port_vol = variance.max(0.0).sqrt();
        let sharpe = if port_vol > 1e-12 { (port_return - rf) / port_vol } else { 0.0 };

        // Apply volatility targeting scaling if requested
        let (final_weights, final_vol, final_ret) = if method == OptimizationMethod::VolatilityTargeted {
            if let Some(target) = constraints.target_volatility.and_then(|v| v.to_f64()) {
                if port_vol > 1e-12 {
                    let scale = target / port_vol;
                    let scaled: Vec<f64> = weights.iter().map(|&w| w * scale).collect();
                    let wv2 = DVector::from_vec(scaled.clone());
                    let var2 = (wv2.transpose() * &sigma_mat * &wv2)[(0, 0)];
                    let ret2 = wv2.dot(&mu_vec);
                    (scaled, var2.max(0.0).sqrt(), ret2)
                } else {
                    (weights, port_vol, port_return)
                }
            } else {
                (weights, port_vol, port_return)
            }
        } else {
            (weights, port_vol, port_return)
        };

        let weight_pairs: Vec<(String, Decimal)> = assets.iter()
            .zip(final_weights.iter())
            .map(|(a, &w)| (a.clone(), f64_to_decimal(w)))
            .collect();

        Ok(OptimizedPortfolio {
            weights: weight_pairs,
            expected_return: f64_to_decimal(final_ret),
            estimated_volatility: f64_to_decimal(final_vol),
            sharpe_ratio: f64_to_decimal(sharpe),
            optimization_method: format!("{:?}", method),
        })
    }
}

// ---------------------------------------------------------------------------
// Optimization algorithms (nalgebra f64 internally)
// ---------------------------------------------------------------------------

/// Minimum variance: w* = Σ⁻¹ 1 / (1ᵀ Σ⁻¹ 1)
fn minimum_variance(sigma: &[f64], n: usize) -> Result<Vec<f64>, String> {
    let mat = DMatrix::from_row_slice(n, n, sigma);
    let inv = mat.clone().try_inverse()
        .ok_or("Covariance matrix is not invertible — check for linear dependence")?;
    let ones = DVector::from_element(n, 1.0_f64);
    let inv_ones = &inv * &ones;
    let denom = ones.dot(&inv_ones);
    if denom.abs() < 1e-12 {
        return Err("Degenerate covariance matrix".to_string());
    }
    Ok(inv_ones.iter().map(|&w| w / denom).collect())
}

/// Maximum Sharpe via golden-section search on target return.
fn maximum_sharpe(mu: &[f64], sigma: &[f64], n: usize, rf: f64) -> Result<Vec<f64>, String> {
    // Tangency portfolio: w* = Σ⁻¹(μ - rf·1) / (1ᵀ Σ⁻¹ (μ - rf·1))
    let mat = DMatrix::from_row_slice(n, n, sigma);
    let inv = mat.clone().try_inverse()
        .ok_or("Covariance matrix is not invertible")?;
    let mu_vec = DVector::from_vec(mu.to_vec());
    let ones = DVector::from_element(n, 1.0_f64);
    let excess = &mu_vec - rf * &ones;
    let inv_excess = &inv * &excess;
    let denom = ones.dot(&inv_excess);
    if denom.abs() < 1e-12 {
        return minimum_variance(sigma, n); // Fallback
    }
    Ok(inv_excess.iter().map(|&w| w / denom).collect())
}

/// Risk budget (equal): iteratively reweight so each asset's marginal contribution = 1/n.
fn risk_budget_equal(sigma: &[f64], n: usize) -> Result<Vec<f64>, String> {
    let mat = DMatrix::from_row_slice(n, n, sigma);
    let target = 1.0 / n as f64;
    let mut w: Vec<f64> = vec![1.0 / n as f64; n];

    // Newton-like iteration (Maillard, Roncalli, Teïletche 2010)
    for _ in 0..200 {
        let wvec = DVector::from_vec(w.clone());
        let sigma_w = &mat * &wvec;
        let port_var = wvec.dot(&sigma_w);
        if port_var < 1e-14 {
            break;
        }
        // Marginal contribution to risk for each asset
        let mrc: Vec<f64> = sigma_w.iter().map(|&sw| sw / port_var.sqrt()).collect();
        // Risk contribution
        let rc: Vec<f64> = w.iter().zip(mrc.iter()).map(|(&wi, &mrci)| wi * mrci).collect();
        let port_risk: f64 = rc.iter().sum::<f64>();

        let mut converged = true;
        for i in 0..n {
            let gradient = rc[i] / port_risk - target;
            if gradient.abs() > 1e-8 {
                converged = false;
            }
            // Simple gradient step
            w[i] *= 1.0 - 0.5 * gradient;
            w[i] = w[i].max(1e-6);
        }
        // Renormalise
        let sum: f64 = w.iter().sum();
        w.iter_mut().for_each(|wi| *wi /= sum);
        if converged { break; }
    }
    Ok(w)
}

/// Volatility targeted: scale max-Sharpe weights to hit target volatility.
fn volatility_targeted(
    mu: &[f64],
    sigma: &[f64],
    n: usize,
    rf: f64,
    target_vol: f64,
) -> Result<Vec<f64>, String> {
    let base = maximum_sharpe(mu, sigma, n, rf)?;
    let mat = DMatrix::from_row_slice(n, n, sigma);
    let wvec = DVector::from_vec(base.clone());
    let variance = (wvec.transpose() * &mat * &wvec)[(0, 0)];
    let port_vol = variance.max(0.0).sqrt();
    if port_vol < 1e-12 {
        return Ok(base);
    }
    let scale = target_vol / port_vol;
    Ok(base.iter().map(|&w| w * scale).collect())
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

fn clip_and_normalise(mut w: Vec<f64>, min_w: f64, max_w: f64, n: usize) -> Vec<f64> {
    // Allow weights below min only if negative (short allowed); for long-only clip at min
    for wi in &mut w {
        if *wi < min_w { *wi = min_w; }
        if *wi > max_w { *wi = max_w; }
    }
    let sum: f64 = w.iter().sum::<f64>();
    if sum.abs() < 1e-12 {
        return vec![1.0 / n as f64; n];
    }
    w.iter().map(|&wi| wi / sum).collect()
}

fn dec_vec_to_f64(v: &[Decimal]) -> Result<Vec<f64>, String> {
    v.iter().map(|d| d.to_f64().ok_or_else(|| format!("Decimal {} out of f64 range", d))).collect()
}

fn dec_matrix_to_f64(m: &[Vec<Decimal>]) -> Result<Vec<f64>, String> {
    let mut flat = Vec::with_capacity(m.len() * m.len());
    for row in m {
        for d in row {
            flat.push(d.to_f64().ok_or_else(|| format!("Decimal {} out of f64 range", d))?);
        }
    }
    Ok(flat)
}

fn f64_to_decimal(v: f64) -> Decimal {
    Decimal::from_f64(v).unwrap_or(Decimal::ZERO).round_dp(8)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple 2-asset case for deterministic testing.
    fn two_asset_setup() -> (Vec<String>, Vec<Decimal>, Vec<Vec<Decimal>>) {
        let assets = vec!["A".to_string(), "B".to_string()];
        let mu = vec![Decimal::new(10, 2), Decimal::new(15, 2)]; // 10%, 15%
        // Cov: [[0.04, 0.01], [0.01, 0.09]]
        let cov = vec![
            vec![Decimal::new(4, 2), Decimal::new(1, 2)],
            vec![Decimal::new(1, 2), Decimal::new(9, 2)],
        ];
        (assets, mu, cov)
    }

    #[test]
    fn test_min_variance_weights_sum_to_one() {
        let (assets, mu, cov) = two_asset_setup();
        let opt = PortfolioOptimizer::new();
        let result = opt.optimize(&assets, &mu, &cov, &OptimizationConstraints::default(), OptimizationMethod::MinimumVariance)
            .expect("failed");
        let weight_sum: Decimal = result.weights.iter().map(|(_, w)| *w).sum();
        let diff = (weight_sum - Decimal::ONE).abs();
        assert!(diff < Decimal::new(1, 6), "weights sum {} != 1", weight_sum);
    }

    #[test]
    fn test_max_sharpe_positive_return() {
        let (assets, mu, cov) = two_asset_setup();
        let opt = PortfolioOptimizer::new();
        let result = opt.optimize(&assets, &mu, &cov, &OptimizationConstraints::default(), OptimizationMethod::MaximumSharpe)
            .expect("failed");
        assert!(result.expected_return > Decimal::ZERO);
        assert!(result.sharpe_ratio > Decimal::ZERO);
    }

    #[test]
    fn test_risk_budget_weights_positive() {
        let (assets, mu, cov) = two_asset_setup();
        let opt = PortfolioOptimizer::new();
        let result = opt.optimize(&assets, &mu, &cov, &OptimizationConstraints::default(), OptimizationMethod::RiskBudget)
            .expect("failed");
        for (_, w) in &result.weights {
            assert!(*w > Decimal::ZERO, "risk budget weight must be positive");
        }
    }

    #[test]
    fn test_empty_assets_error() {
        let opt = PortfolioOptimizer::new();
        let result = opt.optimize(&[], &[], &[], &OptimizationConstraints::default(), OptimizationMethod::EqualWeight);
        assert!(result.is_err());
    }
}
