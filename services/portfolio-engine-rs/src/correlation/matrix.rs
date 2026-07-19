//! Correlation and Covariance Matrix
//!
//! Provides data structures and computation methods for correlation/covariance matrices.
//! Pearson correlation is computed from return time series. Covariance is derived
//! from correlation × individual volatilities. Positive-definiteness validated via Cholesky.

use nalgebra::DMatrix;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorrelationWindow {
    ShortTerm,
    MediumTerm,
    LongTerm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorrelationType {
    Symbol,
    Currency,
    Sector,
    Theme,
}

/// Symmetric correlation matrix stored in row-major flat Vec<Decimal>.
/// Diagonal is always 1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    pub matrix_type: CorrelationType,
    pub window: CorrelationWindow,
    pub identifiers: Vec<String>,
    pub data: Vec<Decimal>,
    pub rows: usize,
    pub cols: usize,
}

/// Symmetric covariance matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CovarianceMatrix {
    pub identifiers: Vec<String>,
    pub data: Vec<Decimal>,
    pub rows: usize,
    pub cols: usize,
}

impl CorrelationMatrix {
    /// Create identity correlation matrix (all off-diagonals = 0).
    pub fn new(
        matrix_type: CorrelationType,
        window: CorrelationWindow,
        identifiers: Vec<String>,
    ) -> Self {
        let size = identifiers.len();
        let mut data = vec![Decimal::ZERO; size * size];
        for i in 0..size {
            data[i * size + i] = Decimal::ONE;
        }
        Self {
            matrix_type,
            window,
            identifiers,
            data,
            rows: size,
            cols: size,
        }
    }

    /// Compute Pearson correlation matrix from historical return series.
    ///
    /// `returns` maps asset identifier → chronological return series.
    /// Returns error if fewer than 2 assets or fewer than 2 observations.
    pub fn from_returns(
        matrix_type: CorrelationType,
        window: CorrelationWindow,
        returns: &HashMap<String, Vec<Decimal>>,
    ) -> Result<Self, String> {
        let identifiers: Vec<String> = {
            let mut v: Vec<String> = returns.keys().cloned().collect();
            v.sort(); // Deterministic ordering
            v
        };
        let n = identifiers.len();
        if n < 2 {
            return Err("Need at least 2 assets to compute correlation".to_string());
        }

        // Find common length
        let min_len = identifiers
            .iter()
            .map(|id| returns[id].len())
            .min()
            .unwrap_or(0);

        if min_len < 2 {
            return Err("Need at least 2 return observations".to_string());
        }

        // Convert to f64 arrays for computation
        let series: Vec<Vec<f64>> = identifiers
            .iter()
            .map(|id| {
                returns[id]
                    .iter()
                    .take(min_len)
                    .map(|d| d.to_f64().unwrap_or(0.0))
                    .collect()
            })
            .collect();

        let mut data = vec![Decimal::ZERO; n * n];
        for i in 0..n {
            for j in 0..n {
                let corr = if i == j {
                    1.0
                } else {
                    pearson_correlation(&series[i], &series[j])
                };
                let val = Decimal::from_f64(corr.clamp(-1.0, 1.0))
                    .unwrap_or(Decimal::ZERO)
                    .round_dp(8);
                data[i * n + j] = val;
            }
        }

        Ok(Self {
            matrix_type,
            window,
            identifiers,
            data,
            rows: n,
            cols: n,
        })
    }

    pub fn get_correlation(&self, idx_a: usize, idx_b: usize) -> Option<Decimal> {
        if idx_a < self.rows && idx_b < self.cols {
            Some(self.data[idx_a * self.cols + idx_b])
        } else {
            None
        }
    }

    pub fn set_correlation(&mut self, idx_a: usize, idx_b: usize, value: Decimal) {
        if idx_a < self.rows && idx_b < self.cols {
            self.data[idx_a * self.cols + idx_b] = value;
            self.data[idx_b * self.cols + idx_a] = value; // Symmetric
        }
    }

    /// Compute covariance matrix: Cov_ij = corr_ij × σ_i × σ_j
    ///
    /// `vols` must be in the same order as `self.identifiers`.
    pub fn to_covariance(&self, vols: &[Decimal]) -> Result<CovarianceMatrix, String> {
        if vols.len() != self.rows {
            return Err(format!(
                "vols length {} != matrix size {}",
                vols.len(),
                self.rows
            ));
        }
        let n = self.rows;
        let mut data = vec![Decimal::ZERO; n * n];
        for i in 0..n {
            for j in 0..n {
                let corr = self.data[i * n + j];
                let cov = corr * vols[i] * vols[j];
                data[i * n + j] = cov.round_dp(10);
            }
        }
        Ok(CovarianceMatrix {
            identifiers: self.identifiers.clone(),
            data,
            rows: n,
            cols: n,
        })
    }

    /// Check if matrix is positive semi-definite via Cholesky decomposition.
    ///
    /// A valid correlation matrix must be PSD. If eigenvalue cleaning has been
    /// applied, this should always return true.
    pub fn is_positive_semi_definite(&self) -> bool {
        let n = self.rows;
        let flat: Vec<f64> = self
            .data
            .iter()
            .map(|d| d.to_f64().unwrap_or(0.0))
            .collect();
        let mat = DMatrix::from_row_slice(n, n, &flat);
        mat.cholesky().is_some()
    }

    /// Apply eigenvalue cleaning (Marcenko-Pastur threshold).
    ///
    /// Removes noise eigenvalues below the Marcenko-Pastur upper bound to produce
    /// a cleaner, more stable correlation matrix for optimization.
    ///
    /// `t_ratio` = n_observations / n_assets (higher = more observations relative to assets)
    pub fn eigenvalue_clean(&self, t_ratio: f64) -> Result<Self, String> {
        let n = self.rows;
        let flat: Vec<f64> = self
            .data
            .iter()
            .map(|d| d.to_f64().unwrap_or(0.0))
            .collect();
        let mat = DMatrix::from_row_slice(n, n, &flat);

        // Symmetric eigendecomposition
        let eig = mat.symmetric_eigen();
        let mut eigenvalues = eig.eigenvalues.clone();
        let eigenvectors = eig.eigenvectors.clone();

        // Marcenko-Pastur upper bound: λ+ = (1 + 1/√t_ratio)²
        let lambda_plus = if t_ratio > 0.0 {
            (1.0 + (1.0 / t_ratio).sqrt()).powi(2)
        } else {
            1.0
        };

        // Replace noise eigenvalues with their mean
        let noise_eigs: Vec<f64> = eigenvalues
            .iter()
            .copied()
            .filter(|&e| e < lambda_plus)
            .collect();
        let noise_mean = if noise_eigs.is_empty() {
            0.0
        } else {
            noise_eigs.iter().sum::<f64>() / noise_eigs.len() as f64
        };

        for e in eigenvalues.iter_mut() {
            if *e < lambda_plus {
                *e = noise_mean;
            }
        }

        // Reconstruct: C = Q Λ Qᵀ
        let lambda_mat = DMatrix::from_diagonal(&eigenvalues);
        let cleaned = &eigenvectors * lambda_mat * eigenvectors.transpose();

        // Renormalise diagonal to 1
        let mut cleaned_data = vec![Decimal::ZERO; n * n];
        for i in 0..n {
            for j in 0..n {
                let mut val = cleaned[(i, j)];
                if i == j {
                    val = 1.0;
                }
                cleaned_data[i * n + j] = Decimal::from_f64(val.clamp(-1.0, 1.0))
                    .unwrap_or(Decimal::ZERO)
                    .round_dp(8);
            }
        }

        Ok(Self {
            matrix_type: self.matrix_type,
            window: self.window,
            identifiers: self.identifiers.clone(),
            data: cleaned_data,
            rows: n,
            cols: n,
        })
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len());
    if n < 2 {
        return 0.0;
    }
    let nf = n as f64;
    let mx = x[..n].iter().sum::<f64>() / nf;
    let my = y[..n].iter().sum::<f64>() / nf;

    let cov = x[..n]
        .iter()
        .zip(y[..n].iter())
        .map(|(&xi, &yi)| (xi - mx) * (yi - my))
        .sum::<f64>();
    let sx = (x[..n].iter().map(|&xi| (xi - mx).powi(2)).sum::<f64>() / nf).sqrt();
    let sy = (y[..n].iter().map(|&yi| (yi - my).powi(2)).sum::<f64>() / nf).sqrt();

    if sx < 1e-12 || sy < 1e-12 {
        0.0
    } else {
        (cov / nf) / (sx * sy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_diagonal() {
        let m = CorrelationMatrix::new(
            CorrelationType::Symbol,
            CorrelationWindow::MediumTerm,
            vec!["A".to_string(), "B".to_string()],
        );
        assert_eq!(m.get_correlation(0, 0), Some(Decimal::ONE));
        assert_eq!(m.get_correlation(1, 1), Some(Decimal::ONE));
        assert_eq!(m.get_correlation(0, 1), Some(Decimal::ZERO));
    }

    #[test]
    fn test_from_returns_symmetric() {
        let mut returns = HashMap::new();
        returns.insert(
            "A".to_string(),
            vec![
                Decimal::new(1, 2),
                Decimal::new(2, 2),
                Decimal::new(-1, 2),
                Decimal::new(3, 2),
                Decimal::new(-2, 2),
            ],
        );
        returns.insert(
            "B".to_string(),
            vec![
                Decimal::new(1, 2),
                Decimal::new(2, 2),
                Decimal::new(-1, 2),
                Decimal::new(3, 2),
                Decimal::new(-2, 2),
            ],
        );
        let m = CorrelationMatrix::from_returns(
            CorrelationType::Symbol,
            CorrelationWindow::MediumTerm,
            &returns,
        )
        .expect("failed");
        // Same series → perfect correlation = 1
        assert!(m
            .get_correlation(0, 1)
            .map(|v| v > Decimal::new(99, 2))
            .unwrap_or(false));
        // Symmetric
        assert_eq!(m.get_correlation(0, 1), m.get_correlation(1, 0));
    }

    #[test]
    fn test_covariance_diagonal() {
        let m = CorrelationMatrix::new(
            CorrelationType::Symbol,
            CorrelationWindow::MediumTerm,
            vec!["A".to_string(), "B".to_string()],
        );
        let vols = vec![Decimal::new(20, 2), Decimal::new(30, 2)]; // 20%, 30%
        let cov = m.to_covariance(&vols).expect("failed");
        // Diagonal: 0.20² = 0.04, 0.30² = 0.09
        assert_eq!(cov.data[0], Decimal::new(4, 2).round_dp(10));
        assert_eq!(cov.data[3], Decimal::new(9, 2).round_dp(10));
    }
}
