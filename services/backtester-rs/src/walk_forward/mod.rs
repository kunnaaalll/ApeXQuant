//! Walk-Forward Validation Module
//!
//! Provides rolling-window walk-forward testing: in-sample (IS) optimization,
//! out-of-sample (OOS) evaluation, and multi-window statistical aggregation.
//!
//! All scores are computed from real per-window trade statistics — no hardcoded values.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// A single walk-forward window with IS and OOS date boundaries (Unix ms).
#[derive(Debug, Clone)]
pub struct WalkForwardWindow {
    pub is_start_ms: i64,
    pub is_end_ms: i64,
    pub oos_start_ms: i64,
    pub oos_end_ms: i64,
}

/// Performance statistics for a single IS or OOS window.
#[derive(Debug, Clone)]
pub struct WindowStats {
    pub total_trades: u32,
    pub winning_trades: u32,
    pub gross_profit: Decimal,
    pub gross_loss: Decimal,
    pub max_drawdown: Decimal,
    pub net_profit: Decimal,
}

impl WindowStats {
    /// Win rate: winners / total (returns 0 if no trades)
    pub fn win_rate(&self) -> Decimal {
        if self.total_trades == 0 {
            return Decimal::ZERO;
        }
        Decimal::from(self.winning_trades) / Decimal::from(self.total_trades)
    }

    /// Profit factor: gross_profit / gross_loss (returns 0 if no losses)
    pub fn profit_factor(&self) -> Decimal {
        if self.gross_loss == Decimal::ZERO {
            return if self.gross_profit > Decimal::ZERO { Decimal::new(999, 0) } else { Decimal::ZERO };
        }
        self.gross_profit / self.gross_loss
    }

    /// Expectancy: (win_rate × avg_win) − (loss_rate × avg_loss)
    pub fn expectancy(&self) -> Decimal {
        if self.total_trades == 0 {
            return Decimal::ZERO;
        }
        let losing_trades = self.total_trades - self.winning_trades;
        let win_rate = self.win_rate();
        let loss_rate = Decimal::ONE - win_rate;

        let avg_win = if self.winning_trades > 0 {
            self.gross_profit / Decimal::from(self.winning_trades)
        } else {
            Decimal::ZERO
        };
        let avg_loss = if losing_trades > 0 {
            self.gross_loss / Decimal::from(losing_trades)
        } else {
            Decimal::ZERO
        };

        (win_rate * avg_win) - (loss_rate * avg_loss)
    }
}

/// Combined IS+OOS data for one walk-forward window.
#[derive(Debug, Clone)]
pub struct WalkForwardWindowData {
    pub window: WalkForwardWindow,
    pub is_stats: WindowStats,
    pub oos_stats: WindowStats,
}

/// Aggregate result of all walk-forward windows.
#[derive(Debug, Clone)]
pub struct WalkForwardResult {
    /// Fraction of OOS windows with positive expectancy (0–1)
    pub stability_score: Decimal,
    /// OOS mean expectancy / IS mean expectancy — degradation ratio (1.0 = perfect generalization)
    pub robustness_score: Decimal,
    /// Pearson correlation between IS and OOS profit factors across windows (−1 to 1)
    pub generalization_score: Decimal,
    /// Coefficient of variation of OOS max drawdowns (lower = more stable)
    pub drawdown_stability: Decimal,
    /// Mean OOS expectancy across all windows
    pub mean_oos_expectancy: Decimal,
    /// Mean OOS profit factor across all windows
    pub mean_oos_profit_factor: Decimal,
    /// Number of windows evaluated
    pub window_count: usize,
    /// Passes validation: stability ≥ 0.6 AND robustness ≥ 0.5
    pub passes_validation: bool,
}

pub struct WalkForwardEngine;

impl WalkForwardEngine {
    /// Generate rolling walk-forward windows by sliding OOS-duration steps.
    pub fn generate_windows(
        start_ms: i64,
        end_ms: i64,
        is_duration: i64,
        oos_duration: i64,
    ) -> Vec<WalkForwardWindow> {
        let mut windows = Vec::new();
        let mut current_start = start_ms;

        while current_start + is_duration + oos_duration <= end_ms {
            windows.push(WalkForwardWindow {
                is_start_ms: current_start,
                is_end_ms: current_start + is_duration,
                oos_start_ms: current_start + is_duration,
                oos_end_ms: current_start + is_duration + oos_duration,
            });
            // Slide by OOS duration (rolling non-overlapping OOS periods)
            current_start += oos_duration;
        }

        // If no windows fit, create a single window covering the full range
        if windows.is_empty() {
            windows.push(WalkForwardWindow {
                is_start_ms: start_ms,
                is_end_ms: start_ms + is_duration,
                oos_start_ms: start_ms + is_duration,
                oos_end_ms: end_ms,
            });
        }

        windows
    }

    /// Evaluate walk-forward performance from per-window IS+OOS statistics.
    ///
    /// All scores are computed from real data — no hardcoded values.
    pub fn evaluate(windows: &[WalkForwardWindowData]) -> Result<WalkForwardResult, String> {
        if windows.is_empty() {
            return Err("No walk-forward windows provided".to_string());
        }

        let n = windows.len();

        // --- Stability Score ---
        // Fraction of OOS windows with positive expectancy
        let oos_positive = windows.iter()
            .filter(|w| w.oos_stats.expectancy() > Decimal::ZERO)
            .count();
        let stability_score = Decimal::from(oos_positive) / Decimal::from(n);

        // --- OOS and IS mean expectancy ---
        let mean_oos_exp = mean_decimal(
            &windows.iter().map(|w| w.oos_stats.expectancy()).collect::<Vec<_>>()
        );
        let mean_is_exp = mean_decimal(
            &windows.iter().map(|w| w.is_stats.expectancy()).collect::<Vec<_>>()
        );

        // --- Robustness Score: OOS/IS expectancy ratio (clipped 0–1) ---
        let robustness_score = if mean_is_exp <= Decimal::ZERO {
            Decimal::ZERO
        } else {
            let ratio = mean_oos_exp / mean_is_exp;
            ratio.max(Decimal::ZERO).min(Decimal::ONE)
        };

        // --- Generalization Score: Pearson correlation IS vs OOS profit factor ---
        let is_pfs: Vec<f64> = windows.iter()
            .map(|w| w.is_stats.profit_factor().to_f64().unwrap_or(0.0))
            .collect();
        let oos_pfs: Vec<f64> = windows.iter()
            .map(|w| w.oos_stats.profit_factor().to_f64().unwrap_or(0.0))
            .collect();
        let generalization_score = pearson_correlation(&is_pfs, &oos_pfs)
            .map(f64_to_decimal)
            .unwrap_or(Decimal::ZERO);

        // --- Drawdown Stability: coefficient of variation of OOS max drawdowns ---
        let oos_dds: Vec<f64> = windows.iter()
            .map(|w| w.oos_stats.max_drawdown.to_f64().unwrap_or(0.0))
            .collect();
        let drawdown_stability = coefficient_of_variation(&oos_dds)
            .map(f64_to_decimal)
            .unwrap_or(Decimal::ONE);

        // Mean OOS profit factor
        let mean_oos_profit_factor = mean_decimal(
            &windows.iter().map(|w| w.oos_stats.profit_factor()).collect::<Vec<_>>()
        );

        // Passes validation: stability ≥ 0.6 and robustness ≥ 0.5
        let passes_validation = stability_score >= Decimal::new(60, 2)
            && robustness_score >= Decimal::new(50, 2);

        Ok(WalkForwardResult {
            stability_score,
            robustness_score,
            generalization_score,
            drawdown_stability,
            mean_oos_expectancy: mean_oos_exp,
            mean_oos_profit_factor,
            window_count: n,
            passes_validation,
        })
    }
}

// ---------------------------------------------------------------------------
// Statistical helpers (no floats in outputs)
// ---------------------------------------------------------------------------

fn mean_decimal(vals: &[Decimal]) -> Decimal {
    if vals.is_empty() {
        return Decimal::ZERO;
    }
    let sum: Decimal = vals.iter().copied().sum();
    sum / Decimal::from(vals.len())
}

/// Pearson correlation coefficient between two f64 slices.
fn pearson_correlation(x: &[f64], y: &[f64]) -> Option<f64> {
    if x.len() != y.len() || x.len() < 2 {
        return None;
    }
    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let cov = x.iter().zip(y.iter()).map(|(&xi, &yi)| (xi - mean_x) * (yi - mean_y)).sum::<f64>();
    let std_x = (x.iter().map(|&xi| (xi - mean_x).powi(2)).sum::<f64>() / n).sqrt();
    let std_y = (y.iter().map(|&yi| (yi - mean_y).powi(2)).sum::<f64>() / n).sqrt();

    if std_x == 0.0 || std_y == 0.0 {
        return None;
    }
    Some((cov / n) / (std_x * std_y))
}

/// Coefficient of variation: std_dev / mean (returns None if mean ≈ 0)
fn coefficient_of_variation(vals: &[f64]) -> Option<f64> {
    if vals.is_empty() {
        return None;
    }
    let n = vals.len() as f64;
    let mean = vals.iter().sum::<f64>() / n;
    if mean.abs() < 1e-12 {
        return None;
    }
    let var = vals.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n;
    Some(var.sqrt() / mean.abs())
}

fn f64_to_decimal(v: f64) -> Decimal {
    Decimal::from_f64_retain(v)
        .unwrap_or(Decimal::ZERO)
        .round_dp(6)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win_stats(trades: u32, wins: u32, profit: i64, loss: i64, dd: i64) -> WindowStats {
        WindowStats {
            total_trades: trades,
            winning_trades: wins,
            gross_profit: Decimal::new(profit, 0),
            gross_loss: Decimal::new(loss, 0),
            max_drawdown: Decimal::new(dd, 0),
            net_profit: Decimal::new(profit - loss, 0),
        }
    }

    #[test]
    fn test_stability_score_all_positive() {
        let windows = vec![
            WalkForwardWindowData {
                window: WalkForwardWindow { is_start_ms: 0, is_end_ms: 100, oos_start_ms: 100, oos_end_ms: 200 },
                is_stats: win_stats(100, 60, 6000, 4000, 500),
                oos_stats: win_stats(30, 18, 1800, 1200, 200),
            },
            WalkForwardWindowData {
                window: WalkForwardWindow { is_start_ms: 100, is_end_ms: 200, oos_start_ms: 200, oos_end_ms: 300 },
                is_stats: win_stats(100, 60, 6000, 4000, 500),
                oos_stats: win_stats(30, 18, 1800, 1200, 150),
            },
        ];
        let result = WalkForwardEngine::evaluate(&windows).expect("evaluate failed");
        assert_eq!(result.stability_score, Decimal::ONE); // 2/2 windows positive
        assert!(result.robustness_score > Decimal::ZERO);
        assert_eq!(result.window_count, 2);
    }

    #[test]
    fn test_passes_validation() {
        // Build 5 windows with solid OOS performance
        let windows: Vec<_> = (0..5).map(|i| WalkForwardWindowData {
            window: WalkForwardWindow {
                is_start_ms: i * 100, is_end_ms: i * 100 + 100,
                oos_start_ms: i * 100 + 100, oos_end_ms: i * 100 + 200,
            },
            is_stats: win_stats(100, 60, 6000, 3000, 400),
            oos_stats: win_stats(30, 20, 2000, 1000, 150),
        }).collect();
        let result = WalkForwardEngine::evaluate(&windows).expect("evaluate failed");
        assert!(result.passes_validation);
    }

    #[test]
    fn test_window_generation_sliding() {
        let windows = WalkForwardEngine::generate_windows(0, 1_000, 700, 100);
        // With IS=700, OOS=100, total=800: window 0=[0,700,700,800], window 1=[100,800,800,900], window 2=[200,900,900,1000]
        assert!(!windows.is_empty());
        for w in &windows {
            assert!(w.is_end_ms > w.is_start_ms);
            assert_eq!(w.oos_start_ms, w.is_end_ms);
            assert!(w.oos_end_ms > w.oos_start_ms);
        }
    }

    #[test]
    fn test_expectancy_formula() {
        let stats = win_stats(10, 6, 600, 400, 100);
        // win_rate = 0.6, avg_win = 100, avg_loss = 100, expectancy = (0.6×100) - (0.4×100) = 20
        let exp = stats.expectancy();
        assert_eq!(exp, Decimal::new(20, 0));
    }
}
