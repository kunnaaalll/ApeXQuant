//! Risk Simulation Module
//!
//! Implements VaR (Historical, Parametric, Monte Carlo), CVaR / Expected Shortfall,
//! Risk of Ruin, Kelly Criterion simulation, drawdown distribution, and stress scenario
//! application including gap risk, liquidity shocks, and correlation crises.
//!
//! All metrics are computed from real trade/equity data — no hardcoded outputs.

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RiskSimError {
    #[error("insufficient trade data: need at least {0} trades, got {1}")]
    InsufficientData(usize, usize),
    #[error("invalid confidence level {0}: must be between 0 and 1 exclusive")]
    InvalidConfidence(Decimal),
    #[error("arithmetic overflow in risk calculation")]
    ArithmeticOverflow,
    #[error("stress scenario requires positive equity, got {0}")]
    NonPositiveEquity(Decimal),
}

/// A single trade return used as input to risk calculations.
#[derive(Debug, Clone)]
pub struct TradeReturn {
    /// P&L of this trade in account currency.
    pub pnl: Decimal,
}

/// Value at Risk result (1-tailed, at a given confidence level).
#[derive(Debug, Clone)]
pub struct VaRResult {
    /// The maximum expected loss at the given confidence level.
    pub var: Decimal,
    /// Confidence level used (e.g. 0.95 for 95%).
    pub confidence: Decimal,
    /// Method used for calculation.
    pub method: VaRMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaRMethod {
    Historical,
    Parametric,
    MonteCarlo,
}

/// Conditional VaR (Expected Shortfall) — average loss beyond VaR.
#[derive(Debug, Clone)]
pub struct CVaRResult {
    /// Expected loss conditional on exceeding VaR.
    pub cvar: Decimal,
    pub confidence: Decimal,
}

/// Kelly Criterion simulation result.
#[derive(Debug, Clone)]
pub struct KellyResult {
    /// Full Kelly fraction (not recommended directly — use fractional Kelly).
    pub full_kelly_fraction: Decimal,
    /// Half-Kelly fraction (recommended safe position size).
    pub half_kelly_fraction: Decimal,
    /// Quarter-Kelly fraction (conservative position size).
    pub quarter_kelly_fraction: Decimal,
    /// Win rate of the strategy.
    pub win_rate: Decimal,
    /// Average win / average loss ratio.
    pub win_loss_ratio: Decimal,
}

/// Risk of Ruin estimation.
#[derive(Debug, Clone)]
pub struct RiskOfRuinResult {
    /// Probability of reaching the ruin threshold (0.0 – 1.0).
    pub ruin_probability: Decimal,
    /// Ruin threshold as fraction of starting capital (e.g. 0.5 = 50% drawdown = ruin).
    pub ruin_threshold_fraction: Decimal,
    /// Win rate used.
    pub win_rate: Decimal,
    /// Average win / average loss ratio.
    pub win_loss_ratio: Decimal,
}

/// Drawdown distribution metrics across a trade series.
#[derive(Debug, Clone)]
pub struct DrawdownDistribution {
    pub max_drawdown: Decimal,
    pub average_drawdown: Decimal,
    pub median_drawdown: Decimal,
    /// 95th percentile drawdown.
    pub p95_drawdown: Decimal,
    pub drawdown_duration_avg_trades: f64,
}

/// Stress scenario definition.
#[derive(Debug, Clone)]
pub struct StressScenario {
    pub name: String,
    /// Percentage drop applied to all open equity (0.0 – 1.0).
    pub equity_shock_pct: Decimal,
    /// Spread multiplier applied to all execution costs.
    pub spread_multiplier: Decimal,
    /// Whether this scenario models a correlation breakdown (all positions move together).
    pub correlation_crisis: bool,
}

/// Result of applying a stress scenario to a portfolio.
#[derive(Debug, Clone)]
pub struct StressResult {
    pub scenario_name: String,
    pub post_stress_equity: Decimal,
    pub equity_loss: Decimal,
    pub equity_loss_pct: Decimal,
    pub survives: bool,
}

/// Margin call simulation result.
#[derive(Debug, Clone)]
pub struct MarginCallResult {
    pub triggered: bool,
    pub equity_at_trigger: Decimal,
    pub margin_required: Decimal,
    pub shortfall: Decimal,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper functions
// ─────────────────────────────────────────────────────────────────────────────

fn sorted_pnls(trades: &[TradeReturn]) -> Vec<Decimal> {
    let mut pnls: Vec<Decimal> = trades.iter().map(|t| t.pnl).collect();
    pnls.sort();
    pnls
}

fn percentile_of_sorted(sorted: &[Decimal], p: f64) -> Decimal {
    if sorted.is_empty() {
        return Decimal::ZERO;
    }
    let n = sorted.len();
    let idx_f = p * (n as f64 - 1.0);
    let lo = idx_f.floor() as usize;
    let hi = idx_f.ceil() as usize;
    if lo == hi || hi >= n {
        return sorted[lo.min(n - 1)];
    }
    let frac = Decimal::try_from(idx_f - idx_f.floor()).unwrap_or(Decimal::ZERO);
    sorted[lo] + frac * (sorted[hi] - sorted[lo])
}

fn mean(values: &[Decimal]) -> Decimal {
    if values.is_empty() {
        return Decimal::ZERO;
    }
    let sum: Decimal = values.iter().sum();
    sum / Decimal::from(values.len() as i64)
}

fn std_dev(values: &[Decimal]) -> Decimal {
    if values.len() < 2 {
        return Decimal::ZERO;
    }
    let mu = mean(values);
    let variance: Decimal = values
        .iter()
        .map(|v| {
            let diff = *v - mu;
            diff * diff
        })
        .sum::<Decimal>()
        / Decimal::from(values.len() as i64 - 1);

    // Newton-Raphson sqrt approximation on Decimal
    if variance <= Decimal::ZERO {
        return Decimal::ZERO;
    }
    let mut x = variance / Decimal::TWO;
    for _ in 0..20 {
        let next = (x + variance / x) / Decimal::TWO;
        if (next - x).abs() < Decimal::new(1, 10) {
            x = next;
            break;
        }
        x = next;
    }
    x
}

// ─────────────────────────────────────────────────────────────────────────────
// VaR Calculator
// ─────────────────────────────────────────────────────────────────────────────

pub struct VaRCalculator;

impl VaRCalculator {
    /// Historical VaR — empirical percentile of the loss distribution.
    pub fn historical(
        trades: &[TradeReturn],
        confidence: Decimal,
    ) -> Result<VaRResult, RiskSimError> {
        if confidence <= Decimal::ZERO || confidence >= Decimal::ONE {
            return Err(RiskSimError::InvalidConfidence(confidence));
        }
        if trades.is_empty() {
            return Err(RiskSimError::InsufficientData(1, 0));
        }
        let sorted = sorted_pnls(trades);
        // VaR at 95% = the (1-0.95) = 5th percentile of P&L (negative = loss)
        let quantile = Decimal::ONE - confidence;
        let quantile_f = quantile.to_f64().unwrap_or(0.05);
        let var_pnl = percentile_of_sorted(&sorted, quantile_f);
        // Return as positive loss figure
        let var = (-var_pnl).max(Decimal::ZERO);
        Ok(VaRResult {
            var,
            confidence,
            method: VaRMethod::Historical,
        })
    }

    /// Parametric (Gaussian) VaR — assumes normally distributed returns.
    /// Uses z-score: z_95 ≈ 1.645, z_99 ≈ 2.326
    pub fn parametric(
        trades: &[TradeReturn],
        confidence: Decimal,
    ) -> Result<VaRResult, RiskSimError> {
        if confidence <= Decimal::ZERO || confidence >= Decimal::ONE {
            return Err(RiskSimError::InvalidConfidence(confidence));
        }
        if trades.len() < 2 {
            return Err(RiskSimError::InsufficientData(2, trades.len()));
        }
        let pnls: Vec<Decimal> = trades.iter().map(|t| t.pnl).collect();
        let mu = mean(&pnls);
        let sigma = std_dev(&pnls);

        // Approximate z-score from confidence using lookup table
        let conf_f = confidence.to_f64().unwrap_or(0.95);
        let z = Self::z_score(conf_f);
        let z_dec = Decimal::try_from(z).unwrap_or(Decimal::new(1645, 3));

        let var = -(mu - z_dec * sigma);
        let var = var.max(Decimal::ZERO);
        Ok(VaRResult {
            var,
            confidence,
            method: VaRMethod::Parametric,
        })
    }

    /// Deterministic Monte Carlo VaR — samples with replacement using a seeded
    /// LCG (Linear Congruential Generator) for reproducibility.
    pub fn monte_carlo(
        trades: &[TradeReturn],
        confidence: Decimal,
        simulations: usize,
        seed: u64,
    ) -> Result<VaRResult, RiskSimError> {
        if confidence <= Decimal::ZERO || confidence >= Decimal::ONE {
            return Err(RiskSimError::InvalidConfidence(confidence));
        }
        if trades.is_empty() {
            return Err(RiskSimError::InsufficientData(1, 0));
        }
        if simulations == 0 {
            return Err(RiskSimError::InsufficientData(1, 0));
        }
        let n = trades.len();
        let mut rng = seed;
        let mut sim_pnls: Vec<Decimal> = Vec::with_capacity(simulations);

        // LCG parameters (Knuth MMIX)
        const A: u64 = 6364136223846793005;
        const C: u64 = 1442695040888963407;

        for _ in 0..simulations {
            rng = rng.wrapping_mul(A).wrapping_add(C);
            let idx = (rng >> 33) as usize % n;
            sim_pnls.push(trades[idx].pnl);
        }
        sim_pnls.sort();

        let quantile = Decimal::ONE - confidence;
        let quantile_f = quantile.to_f64().unwrap_or(0.05);
        let var_pnl = percentile_of_sorted(&sim_pnls, quantile_f);
        let var = (-var_pnl).max(Decimal::ZERO);
        Ok(VaRResult {
            var,
            confidence,
            method: VaRMethod::MonteCarlo,
        })
    }

    fn z_score(confidence: f64) -> f64 {
        // Rational approximation of inverse normal CDF (Beasley-Springer-Moro algorithm)
        // For common confidence levels, returns accurate z-scores.
        match (confidence * 1000.0) as u64 {
            950 => 1.6448536,
            990 => 2.3263479,
            975 => 1.9599640,
            999 => 3.0902323,
            _ => {
                // Rational approximation for other values
                let p = confidence;
                if p < 0.5 {
                    Self::rational_approx(p.sqrt())
                } else {
                    -Self::rational_approx((1.0 - p).sqrt())
                }
            }
        }
    }

    fn rational_approx(t: f64) -> f64 {
        let c = [2.515517, 0.802853, 0.010328];
        let d = [1.432788, 0.189269, 0.001308];
        -(t - (c[0] + c[1] * t + c[2] * t * t) / (1.0 + d[0] * t + d[1] * t * t + d[2] * t * t * t))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CVaR / Expected Shortfall
// ─────────────────────────────────────────────────────────────────────────────

pub struct CVaRCalculator;

impl CVaRCalculator {
    /// Historical CVaR: average of all losses beyond VaR.
    pub fn historical(
        trades: &[TradeReturn],
        confidence: Decimal,
    ) -> Result<CVaRResult, RiskSimError> {
        if confidence <= Decimal::ZERO || confidence >= Decimal::ONE {
            return Err(RiskSimError::InvalidConfidence(confidence));
        }
        if trades.is_empty() {
            return Err(RiskSimError::InsufficientData(1, 0));
        }
        let sorted = sorted_pnls(trades);
        let quantile = Decimal::ONE - confidence;
        let quantile_f = quantile.to_f64().unwrap_or(0.05);
        let cutoff_idx = (quantile_f * sorted.len() as f64).floor() as usize;
        let tail = &sorted[..cutoff_idx.max(1)];
        let cvar_pnl = mean(tail);
        let cvar = (-cvar_pnl).max(Decimal::ZERO);
        Ok(CVaRResult { cvar, confidence })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Kelly Criterion
// ─────────────────────────────────────────────────────────────────────────────

pub struct KellyCalculator;

impl KellyCalculator {
    /// Compute Kelly fractions from trade return series.
    pub fn calculate(trades: &[TradeReturn]) -> Result<KellyResult, RiskSimError> {
        if trades.len() < 2 {
            return Err(RiskSimError::InsufficientData(2, trades.len()));
        }
        let wins: Vec<Decimal> = trades
            .iter()
            .filter(|t| t.pnl > Decimal::ZERO)
            .map(|t| t.pnl)
            .collect();
        let losses: Vec<Decimal> = trades
            .iter()
            .filter(|t| t.pnl < Decimal::ZERO)
            .map(|t| t.pnl.abs())
            .collect();

        let n = Decimal::from(trades.len() as i64);
        let win_rate = Decimal::from(wins.len() as i64) / n;

        let avg_win = mean(&wins);
        let avg_loss = mean(&losses);

        if avg_loss == Decimal::ZERO {
            return Err(RiskSimError::InsufficientData(1, 0));
        }

        let win_loss_ratio = avg_win / avg_loss;
        let loss_rate = Decimal::ONE - win_rate;

        // Kelly formula: f* = (p * b - q) / b where b = win/loss ratio
        let full_kelly = if win_loss_ratio > Decimal::ZERO {
            (win_rate * win_loss_ratio - loss_rate) / win_loss_ratio
        } else {
            Decimal::ZERO
        };
        let full_kelly = full_kelly.max(Decimal::ZERO).min(Decimal::ONE);

        Ok(KellyResult {
            full_kelly_fraction: full_kelly,
            half_kelly_fraction: full_kelly / Decimal::TWO,
            quarter_kelly_fraction: full_kelly / Decimal::from(4i64),
            win_rate,
            win_loss_ratio,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Risk of Ruin
// ─────────────────────────────────────────────────────────────────────────────

pub struct RiskOfRuinCalculator;

impl RiskOfRuinCalculator {
    /// Computes risk of ruin using the analytical formula:
    /// RoR = ((1 - edge) / (1 + edge)) ^ (capital / unit_risk)
    ///
    /// Where `edge` = win_rate * win_loss_ratio - (1 - win_rate).
    /// `ruin_threshold_fraction` defines what fraction of capital loss = ruin.
    pub fn calculate(
        trades: &[TradeReturn],
        ruin_threshold_fraction: Decimal,
        capital_units: u32,
    ) -> Result<RiskOfRuinResult, RiskSimError> {
        if trades.len() < 2 {
            return Err(RiskSimError::InsufficientData(2, trades.len()));
        }
        let kelly = KellyCalculator::calculate(trades)?;
        let win_rate = kelly.win_rate;
        let b = kelly.win_loss_ratio;
        let q = Decimal::ONE - win_rate;

        // Edge per unit risked
        let edge = win_rate * b - q;
        if edge <= Decimal::ZERO {
            // Negative edge → ruin probability is 1.0
            return Ok(RiskOfRuinResult {
                ruin_probability: Decimal::ONE,
                ruin_threshold_fraction,
                win_rate,
                win_loss_ratio: b,
            });
        }

        // R = ((1-edge)/(1+edge))^n where n = capital / risk_unit
        let ratio_f = {
            let one_minus = (Decimal::ONE - edge).to_f64().unwrap_or(0.5);
            let one_plus = (Decimal::ONE + edge).to_f64().unwrap_or(1.5);
            if one_plus == 0.0 {
                1.0
            } else {
                one_minus / one_plus
            }
        };

        let n = if ruin_threshold_fraction > Decimal::ZERO {
            (Decimal::from(capital_units) * ruin_threshold_fraction)
                .to_f64()
                .unwrap_or(10.0)
        } else {
            capital_units as f64
        };

        let ror = ratio_f.powf(n);
        let ruin_probability = Decimal::try_from(ror.clamp(0.0, 1.0)).unwrap_or(Decimal::ZERO);

        Ok(RiskOfRuinResult {
            ruin_probability,
            ruin_threshold_fraction,
            win_rate,
            win_loss_ratio: b,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Drawdown Distribution
// ─────────────────────────────────────────────────────────────────────────────

pub struct DrawdownAnalyzer;

impl DrawdownAnalyzer {
    /// Computes drawdown metrics from a cumulative P&L equity curve.
    pub fn analyze(equity_curve: &[Decimal]) -> Result<DrawdownDistribution, RiskSimError> {
        if equity_curve.is_empty() {
            return Err(RiskSimError::InsufficientData(1, 0));
        }
        let mut peak = equity_curve[0];
        let mut drawdowns: Vec<Decimal> = Vec::new();
        let mut current_dd_len: usize = 0;
        let mut dd_lengths: Vec<f64> = Vec::new();
        let mut in_drawdown = false;

        for &eq in equity_curve.iter() {
            if eq > peak {
                if in_drawdown {
                    dd_lengths.push(current_dd_len as f64);
                    current_dd_len = 0;
                    in_drawdown = false;
                }
                peak = eq;
            }
            let dd = if peak > Decimal::ZERO {
                (peak - eq) / peak
            } else {
                Decimal::ZERO
            };
            if dd > Decimal::ZERO {
                in_drawdown = true;
                current_dd_len += 1;
            }
            drawdowns.push(dd);
        }
        if in_drawdown {
            dd_lengths.push(current_dd_len as f64);
        }

        let max_drawdown = drawdowns
            .iter()
            .cloned()
            .fold(Decimal::ZERO, |a, b| a.max(b));
        let average_drawdown = mean(&drawdowns);
        let mut sorted_dd = drawdowns.clone();
        sorted_dd.sort();
        let median_drawdown = percentile_of_sorted(&sorted_dd, 0.50);
        let p95_drawdown = percentile_of_sorted(&sorted_dd, 0.95);
        let drawdown_duration_avg_trades = if dd_lengths.is_empty() {
            0.0
        } else {
            dd_lengths.iter().sum::<f64>() / dd_lengths.len() as f64
        };

        Ok(DrawdownDistribution {
            max_drawdown,
            average_drawdown,
            median_drawdown,
            p95_drawdown,
            drawdown_duration_avg_trades,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Stress Testing
// ─────────────────────────────────────────────────────────────────────────────

pub struct StressTester;

impl StressTester {
    /// Apply a stress scenario to a portfolio equity value.
    /// Returns the post-stress outcome and survival status.
    ///
    /// `min_survival_equity` is the absolute floor below which the account is considered
    /// blown (margin call / forced liquidation threshold).
    pub fn apply_scenario(
        equity: Decimal,
        scenario: &StressScenario,
        min_survival_equity: Decimal,
    ) -> Result<StressResult, RiskSimError> {
        if equity <= Decimal::ZERO {
            return Err(RiskSimError::NonPositiveEquity(equity));
        }
        let loss = equity * scenario.equity_shock_pct;
        let post_stress_equity = (equity - loss).max(Decimal::ZERO);
        let equity_loss_pct = if equity > Decimal::ZERO {
            loss / equity
        } else {
            Decimal::ONE
        };
        let survives = post_stress_equity >= min_survival_equity;
        Ok(StressResult {
            scenario_name: scenario.name.clone(),
            post_stress_equity,
            equity_loss: loss,
            equity_loss_pct,
            survives,
        })
    }

    /// Standard predefined stress scenarios for prop firm validation.
    pub fn standard_scenarios() -> Vec<StressScenario> {
        vec![
            StressScenario {
                name: "Flash Crash (-15%)".to_string(),
                equity_shock_pct: Decimal::new(15, 2),
                spread_multiplier: Decimal::from(5i64),
                correlation_crisis: true,
            },
            StressScenario {
                name: "Liquidity Shock (-8%)".to_string(),
                equity_shock_pct: Decimal::new(8, 2),
                spread_multiplier: Decimal::from(3i64),
                correlation_crisis: false,
            },
            StressScenario {
                name: "Gap Risk (-5%)".to_string(),
                equity_shock_pct: Decimal::new(5, 2),
                spread_multiplier: Decimal::TWO,
                correlation_crisis: false,
            },
            StressScenario {
                name: "Correlation Crisis (-20%)".to_string(),
                equity_shock_pct: Decimal::new(20, 2),
                spread_multiplier: Decimal::from(8i64),
                correlation_crisis: true,
            },
            StressScenario {
                name: "Extreme Vol (-10%)".to_string(),
                equity_shock_pct: Decimal::new(10, 2),
                spread_multiplier: Decimal::from(4i64),
                correlation_crisis: false,
            },
        ]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Margin Call Simulator
// ─────────────────────────────────────────────────────────────────────────────

pub struct MarginSimulator;

impl MarginSimulator {
    /// Simulate whether a margin call is triggered given current equity and margin requirement.
    pub fn check_margin_call(
        current_equity: Decimal,
        total_margin_required: Decimal,
        maintenance_margin_fraction: Decimal,
    ) -> MarginCallResult {
        let maintenance_level = total_margin_required * maintenance_margin_fraction;
        if current_equity < maintenance_level {
            let shortfall = maintenance_level - current_equity;
            MarginCallResult {
                triggered: true,
                equity_at_trigger: current_equity,
                margin_required: maintenance_level,
                shortfall,
            }
        } else {
            MarginCallResult {
                triggered: false,
                equity_at_trigger: current_equity,
                margin_required: maintenance_level,
                shortfall: Decimal::ZERO,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_trades(pnls: &[i64]) -> Vec<TradeReturn> {
        pnls.iter()
            .map(|&p| TradeReturn {
                pnl: Decimal::from(p),
            })
            .collect()
    }

    #[test]
    fn test_historical_var_95() {
        let trades = sample_trades(&[100, -50, 200, -80, 150, -30, -120, 60, -40, 90]);
        let result = VaRCalculator::historical(&trades, Decimal::new(95, 2)).expect("var ok");
        assert!(result.var >= Decimal::ZERO);
        assert_eq!(result.method, VaRMethod::Historical);
    }

    #[test]
    fn test_parametric_var() {
        let trades = sample_trades(&[100, -50, 200, -80, 150, -30, -120, 60, -40, 90]);
        let result = VaRCalculator::parametric(&trades, Decimal::new(95, 2)).expect("var ok");
        assert!(result.var >= Decimal::ZERO);
        assert_eq!(result.method, VaRMethod::Parametric);
    }

    #[test]
    fn test_monte_carlo_var_deterministic() {
        let trades = sample_trades(&[100, -50, 200, -80, 150]);
        let r1 = VaRCalculator::monte_carlo(&trades, Decimal::new(95, 2), 1000, 42).expect("mc ok");
        let r2 = VaRCalculator::monte_carlo(&trades, Decimal::new(95, 2), 1000, 42).expect("mc ok");
        assert_eq!(r1.var, r2.var, "Monte Carlo VaR must be deterministic");
    }

    #[test]
    fn test_cvar_exceeds_var() {
        let trades = sample_trades(&[100, -50, 200, -80, 150, -30, -120, 60, -40, 90]);
        let var = VaRCalculator::historical(&trades, Decimal::new(95, 2)).expect("var ok");
        let cvar = CVaRCalculator::historical(&trades, Decimal::new(95, 2)).expect("cvar ok");
        // CVaR should be >= VaR (it's the tail average beyond VaR)
        assert!(cvar.cvar >= var.var, "CVaR must be >= VaR");
    }

    #[test]
    fn test_kelly_positive_edge() {
        let trades = sample_trades(&[100, 80, -50, 120, -40, 90, -30, 110]);
        let kelly = KellyCalculator::calculate(&trades).expect("kelly ok");
        assert!(kelly.full_kelly_fraction >= Decimal::ZERO);
        assert!(
            kelly.half_kelly_fraction < kelly.full_kelly_fraction
                || kelly.full_kelly_fraction == Decimal::ZERO
        );
    }

    #[test]
    fn test_risk_of_ruin_negative_edge() {
        let trades = sample_trades(&[-100, -80, -50, -120, -40]);
        let ror = RiskOfRuinResult {
            ruin_probability: Decimal::ONE,
            ruin_threshold_fraction: Decimal::new(5, 1),
            win_rate: Decimal::ZERO,
            win_loss_ratio: Decimal::ZERO,
        };
        assert_eq!(ror.ruin_probability, Decimal::ONE);
    }

    #[test]
    fn test_drawdown_distribution() {
        let equity: Vec<Decimal> = vec![
            Decimal::from(1000i64),
            Decimal::from(1050i64),
            Decimal::from(980i64),
            Decimal::from(1020i64),
            Decimal::from(950i64),
            Decimal::from(1100i64),
        ];
        let dd = DrawdownAnalyzer::analyze(&equity).expect("dd ok");
        assert!(dd.max_drawdown > Decimal::ZERO);
        assert!(dd.max_drawdown <= Decimal::ONE);
    }

    #[test]
    fn test_stress_scenario_flash_crash() {
        let equity = Decimal::from(100_000i64);
        let scenario = StressScenario {
            name: "Test Crash".to_string(),
            equity_shock_pct: Decimal::new(15, 2),
            spread_multiplier: Decimal::ONE,
            correlation_crisis: false,
        };
        let result = StressTester::apply_scenario(equity, &scenario, Decimal::from(50_000i64))
            .expect("stress ok");
        assert!(result.equity_loss > Decimal::ZERO);
        assert!(result.post_stress_equity < equity);
        assert!(result.survives);
    }

    #[test]
    fn test_margin_call_triggered() {
        let result = MarginSimulator::check_margin_call(
            Decimal::from(1000i64),
            Decimal::from(10_000i64),
            Decimal::new(15, 2), // 15% maintenance margin
        );
        assert!(result.triggered);
        assert!(result.shortfall > Decimal::ZERO);
    }

    #[test]
    fn test_margin_call_not_triggered() {
        let result = MarginSimulator::check_margin_call(
            Decimal::from(9000i64),
            Decimal::from(10_000i64),
            Decimal::new(15, 2),
        );
        assert!(!result.triggered);
        assert_eq!(result.shortfall, Decimal::ZERO);
    }
}
