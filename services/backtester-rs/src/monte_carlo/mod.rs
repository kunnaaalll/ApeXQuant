//! Monte Carlo Simulation Module
//!
//! Deterministic trade sequence reshuffling, drawdown probability, and risk of ruin.
//! Uses ChaCha8Rng (seeded) for byte-perfect deterministic replay.
//!
//! All financial outputs use `rust_decimal::Decimal`. Internal simulation uses
//! f64 via `rand` / `statrs` only for statistical sampling, converted on exit.

use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

/// A single historical trade result used as simulation input.
#[derive(Debug, Clone)]
pub struct TradeResult {
    /// Net PnL in account currency (positive = win, negative = loss)
    pub pnl: Decimal,
}

/// Parameters for a Monte Carlo simulation run.
#[derive(Debug, Clone)]
pub struct MonteCarloParams {
    /// Starting account equity
    pub initial_equity: Decimal,
    /// Ruin threshold — equity level considered "account blown"
    pub ruin_threshold: Decimal,
    /// Number of simulation iterations (recommended: 10_000)
    pub num_simulations: usize,
    /// Deterministic seed — same seed produces identical results
    pub seed: u64,
}

impl Default for MonteCarloParams {
    fn default() -> Self {
        Self {
            initial_equity: Decimal::new(10_000, 0),
            ruin_threshold: Decimal::new(5_000, 0),
            num_simulations: 10_000,
            seed: 42,
        }
    }
}

/// Per-simulation outcome.
#[derive(Debug, Clone)]
struct SimRun {
    final_equity: f64,
    max_drawdown: f64,
    reached_ruin: bool,
}

/// Full Monte Carlo result — all values in Decimal.
#[derive(Debug, Clone)]
pub struct MonteCarloResult {
    /// Seed used — log this for replay audit
    pub seed: u64,
    /// Number of completed simulation runs
    pub num_simulations: usize,
    /// Number of trades in each resampled path
    pub path_length: usize,

    // --- Drawdown distribution ---
    /// Median (P50) maximum drawdown across all runs
    pub median_max_drawdown: Decimal,
    /// 5th percentile max drawdown (mild scenario)
    pub p05_max_drawdown: Decimal,
    /// 25th percentile max drawdown
    pub p25_max_drawdown: Decimal,
    /// 75th percentile max drawdown
    pub p75_max_drawdown: Decimal,
    /// 95th percentile max drawdown (severe scenario)
    pub p95_max_drawdown: Decimal,
    /// 99th percentile max drawdown (tail event)
    pub p99_max_drawdown: Decimal,

    // --- Return distribution ---
    /// Median final equity across all runs
    pub median_final_equity: Decimal,
    /// 5th percentile final equity
    pub p05_final_equity: Decimal,
    /// 95th percentile final equity
    pub p95_final_equity: Decimal,

    // --- Risk metrics ---
    /// Fraction of runs ending in ruin (equity ≤ ruin_threshold)
    pub risk_of_ruin: Decimal,
    /// P(max_drawdown > 20% of initial equity)
    pub prob_drawdown_exceeds_20pct: Decimal,
    /// P(max_drawdown > 40% of initial equity)
    pub prob_drawdown_exceeds_40pct: Decimal,
    /// Expected max drawdown (mean across runs)
    pub expected_max_drawdown: Decimal,
}

pub struct MonteCarloEngine;

impl MonteCarloEngine {
    /// Run Monte Carlo simulation on a historical trade sequence.
    ///
    /// Each simulation run randomly resamples (with replacement) from the provided
    /// trade history and replays the equity curve. ChaCha8Rng ensures byte-perfect
    /// reproducibility given the same seed.
    pub fn run(
        trades: &[TradeResult],
        params: &MonteCarloParams,
    ) -> Result<MonteCarloResult, String> {
        if trades.is_empty() {
            return Err("Monte Carlo requires at least 1 trade in history".to_string());
        }
        if params.num_simulations == 0 {
            return Err("num_simulations must be > 0".to_string());
        }

        let initial_equity_f64 = params
            .initial_equity
            .to_f64()
            .ok_or("initial_equity out of f64 range")?;
        let ruin_threshold_f64 = params
            .ruin_threshold
            .to_f64()
            .ok_or("ruin_threshold out of f64 range")?;
        let path_length = trades.len();

        // Convert PnL to f64 for simulation math
        let pnl_f64: Vec<f64> = trades
            .iter()
            .map(|t| t.pnl.to_f64().unwrap_or(0.0))
            .collect();

        // Generate per-simulation seeds from root seed (deterministic)
        // Each simulation gets its own sub-seed derived from root + index
        let runs: Vec<SimRun> = (0..params.num_simulations)
            .into_par_iter()
            .map(|sim_idx| {
                // Derive deterministic sub-seed: combine root seed with simulation index
                let sub_seed = params
                    .seed
                    .wrapping_add((sim_idx as u64).wrapping_mul(6_364_136_223_846_793_005));
                let mut rng = ChaCha8Rng::seed_from_u64(sub_seed);

                // Bootstrap: sample path_length trades with replacement
                let sampled: Vec<f64> = (0..path_length)
                    .map(|_| {
                        // Uniform random index into pnl_f64
                        let idx = (rng.next_u64() as usize) % pnl_f64.len();
                        pnl_f64[idx]
                    })
                    .collect();

                // Compute equity curve and max drawdown
                let (max_drawdown, final_equity) =
                    compute_equity_curve(initial_equity_f64, &sampled);
                let reached_ruin = final_equity <= ruin_threshold_f64;

                SimRun {
                    final_equity,
                    max_drawdown,
                    reached_ruin,
                }
            })
            .collect();

        // Compute statistics over all runs
        let mut drawdowns: Vec<f64> = runs.iter().map(|r| r.max_drawdown).collect();
        let mut equities: Vec<f64> = runs.iter().map(|r| r.final_equity).collect();
        drawdowns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        equities.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = runs.len() as f64;
        let ruin_count = runs.iter().filter(|r| r.reached_ruin).count();
        let risk_of_ruin = ruin_count as f64 / n;

        let dd20 = initial_equity_f64 * 0.20;
        let dd40 = initial_equity_f64 * 0.40;
        let prob_dd20 = drawdowns.iter().filter(|&&d| d > dd20).count() as f64 / n;
        let prob_dd40 = drawdowns.iter().filter(|&&d| d > dd40).count() as f64 / n;
        let expected_dd = drawdowns.iter().sum::<f64>() / n;

        Ok(MonteCarloResult {
            seed: params.seed,
            num_simulations: params.num_simulations,
            path_length,
            median_max_drawdown: f64_to_decimal(percentile(&drawdowns, 0.50)),
            p05_max_drawdown: f64_to_decimal(percentile(&drawdowns, 0.05)),
            p25_max_drawdown: f64_to_decimal(percentile(&drawdowns, 0.25)),
            p75_max_drawdown: f64_to_decimal(percentile(&drawdowns, 0.75)),
            p95_max_drawdown: f64_to_decimal(percentile(&drawdowns, 0.95)),
            p99_max_drawdown: f64_to_decimal(percentile(&drawdowns, 0.99)),
            median_final_equity: f64_to_decimal(percentile(&equities, 0.50)),
            p05_final_equity: f64_to_decimal(percentile(&equities, 0.05)),
            p95_final_equity: f64_to_decimal(percentile(&equities, 0.95)),
            risk_of_ruin: f64_to_decimal(risk_of_ruin),
            prob_drawdown_exceeds_20pct: f64_to_decimal(prob_dd20),
            prob_drawdown_exceeds_40pct: f64_to_decimal(prob_dd40),
            expected_max_drawdown: f64_to_decimal(expected_dd),
        })
    }

    /// Run random permutation test to compute p-value for strategy Sharpe.
    ///
    /// Null hypothesis: the observed Sharpe is due to luck (random ordering of the
    /// same returns). Returns p-value = P(permuted_sharpe >= observed_sharpe).
    pub fn permutation_test(
        trades: &[TradeResult],
        observed_sharpe: Decimal,
        num_permutations: usize,
        seed: u64,
    ) -> Result<Decimal, String> {
        if trades.len() < 2 {
            return Err("Need at least 2 trades for permutation test".to_string());
        }
        let obs_f64 = observed_sharpe.to_f64().ok_or("Sharpe out of range")?;
        let pnls: Vec<f64> = trades
            .iter()
            .map(|t| t.pnl.to_f64().unwrap_or(0.0))
            .collect();

        let exceed_count: usize = (0..num_permutations)
            .into_par_iter()
            .filter(|&i| {
                let sub_seed =
                    seed.wrapping_add((i as u64).wrapping_mul(2_862_933_555_777_941_757));
                let mut rng = ChaCha8Rng::seed_from_u64(sub_seed);
                let mut shuffled = pnls.clone();
                shuffled.shuffle(&mut rng);
                let s = sharpe_ratio(&shuffled);
                s >= obs_f64
            })
            .count();

        let p_value = exceed_count as f64 / num_permutations as f64;
        Ok(f64_to_decimal(p_value))
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Compute equity curve from initial equity and a PnL sequence.
/// Returns (max_drawdown_in_dollars, final_equity).
fn compute_equity_curve(initial_equity: f64, pnls: &[f64]) -> (f64, f64) {
    let mut equity = initial_equity;
    let mut peak = initial_equity;
    let mut max_drawdown: f64 = 0.0;

    for &pnl in pnls {
        equity += pnl;
        if equity > peak {
            peak = equity;
        }
        let dd = peak - equity;
        if dd > max_drawdown {
            max_drawdown = dd;
        }
    }
    (max_drawdown, equity)
}

/// Linear interpolation percentile on a sorted slice (0.0 = min, 1.0 = max).
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let idx = p * (sorted.len() - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    if lo == hi {
        return sorted[lo];
    }
    let frac = idx - lo as f64;
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}

/// Annualised Sharpe ratio (mean/std of returns, √252 scaling for daily).
fn sharpe_ratio(pnls: &[f64]) -> f64 {
    if pnls.len() < 2 {
        return 0.0;
    }
    let n = pnls.len() as f64;
    let mean = pnls.iter().sum::<f64>() / n;
    let var = pnls.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std = var.sqrt();
    if std == 0.0 {
        return 0.0;
    }
    (mean / std) * 252_f64.sqrt()
}

/// Convert f64 to Decimal, rounding to 6 decimal places.
fn f64_to_decimal(v: f64) -> Decimal {
    Decimal::from_f64_retain(v)
        .unwrap_or(Decimal::ZERO)
        .round_dp(6)
}

// Helper to get next random u64 without importing full rng trait separately
trait NextU64 {
    fn next_u64(&mut self) -> u64;
}

impl NextU64 for ChaCha8Rng {
    fn next_u64(&mut self) -> u64 {
        use rand::RngCore;
        RngCore::next_u64(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trades(pnls: &[i64]) -> Vec<TradeResult> {
        pnls.iter()
            .map(|&p| TradeResult {
                pnl: Decimal::new(p, 0),
            })
            .collect()
    }

    #[test]
    fn test_monte_carlo_deterministic() {
        let trades = make_trades(&[100, -50, 200, -80, 150, -30, 120, -60, 90, -40]);
        let params = MonteCarloParams {
            initial_equity: Decimal::new(10_000, 0),
            ruin_threshold: Decimal::new(5_000, 0),
            num_simulations: 1_000,
            seed: 12345,
        };
        let r1 = MonteCarloEngine::run(&trades, &params).expect("run 1 failed");
        let r2 = MonteCarloEngine::run(&trades, &params).expect("run 2 failed");
        // Determinism: both runs must produce identical results
        assert_eq!(r1.median_max_drawdown, r2.median_max_drawdown);
        assert_eq!(r1.risk_of_ruin, r2.risk_of_ruin);
        assert_eq!(r1.p95_max_drawdown, r2.p95_max_drawdown);
    }

    #[test]
    fn test_risk_of_ruin_bounds() {
        let trades = make_trades(&[100, -50, 200, -80]);
        let params = MonteCarloParams::default();
        let result = MonteCarloEngine::run(&trades, &params).expect("failed");
        assert!(result.risk_of_ruin >= Decimal::ZERO);
        assert!(result.risk_of_ruin <= Decimal::ONE);
    }

    #[test]
    fn test_empty_trades_error() {
        let result = MonteCarloEngine::run(&[], &MonteCarloParams::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_permutation_test_p_value_bounds() {
        let trades = make_trades(&[100, -50, 200, -80, 150]);
        let p = MonteCarloEngine::permutation_test(
            &trades,
            Decimal::new(15, 1), // 1.5 Sharpe
            1000,
            99,
        )
        .expect("failed");
        assert!(p >= Decimal::ZERO);
        assert!(p <= Decimal::ONE);
    }
}
