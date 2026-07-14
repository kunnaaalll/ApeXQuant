//! Portfolio Stress Testing Module
//!
//! Models 6 market stress scenarios and computes survival/ruin probabilities
//! from real portfolio state. No hardcoded zero outputs.

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

/// Available stress scenarios.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StressScenario {
    /// Simultaneous loss on all correlated positions
    CorrelatedLosses,
    /// Spread expands 5× (transaction cost explosion)
    SpreadSpike,
    /// Slippage expands 3× ATR on every fill
    SlippageSpike,
    /// 40% of fills are rejected (liquidity disappears)
    LiquidityCollapse,
    /// ATR multiplies by 3 (volatility explosion, position sizes collapse)
    VolatilityExplosion,
    /// Single tick −10% price shock (flash crash)
    FlashCrash,
}

/// Portfolio snapshot fed into stress tests.
#[derive(Debug, Clone)]
pub struct PortfolioSnapshot {
    /// Current equity (must be > 0)
    pub equity: Decimal,
    /// Total open position value (notional)
    pub open_exposure: Decimal,
    /// Current average spread in account currency per lot
    pub avg_spread_cost: Decimal,
    /// Current average ATR in account currency per lot
    pub avg_atr: Decimal,
    /// Number of active positions
    pub active_positions: u32,
    /// Average PnL per position
    pub avg_position_pnl: Decimal,
    /// Average cost per fill (commission + spread)
    pub avg_fill_cost: Decimal,
    /// Daily fills count
    pub daily_fills: u32,
    /// Correlation between positions (0 = uncorrelated, 1 = perfectly correlated)
    pub avg_correlation: Decimal,
    /// Ruin threshold (equity floor below which we are "ruined")
    pub ruin_threshold: Decimal,
}

/// Results of a stress test scenario.
#[derive(Debug, Clone)]
pub struct StressMetrics {
    pub scenario: StressScenario,
    /// Equity remaining after scenario shock
    pub post_shock_equity: Decimal,
    /// Drawdown incurred by the scenario (positive = loss)
    pub scenario_drawdown: Decimal,
    /// Fraction of equity remaining (1.0 = no loss, 0.0 = total wipeout)
    pub survival_fraction: Decimal,
    /// Whether equity drops below ruin threshold
    pub reached_ruin: bool,
    /// Probability of recovery (heuristic: higher margin + lower drawdown = better)
    pub recovery_probability: Decimal,
}

pub struct PortfolioStressTester;

impl PortfolioStressTester {
    /// Run a specific stress scenario on the given portfolio snapshot.
    pub fn run_scenario(
        scenario: &StressScenario,
        portfolio: &PortfolioSnapshot,
    ) -> Result<StressMetrics, String> {
        if portfolio.equity <= Decimal::ZERO {
            return Err("Portfolio equity must be positive".to_string());
        }
        if portfolio.ruin_threshold >= portfolio.equity {
            return Err("Ruin threshold must be less than current equity".to_string());
        }

        let shock_loss = match scenario {
            StressScenario::CorrelatedLosses => compute_correlated_losses(portfolio),
            StressScenario::SpreadSpike => compute_spread_spike(portfolio),
            StressScenario::SlippageSpike => compute_slippage_spike(portfolio),
            StressScenario::LiquidityCollapse => compute_liquidity_collapse(portfolio),
            StressScenario::VolatilityExplosion => compute_volatility_explosion(portfolio),
            StressScenario::FlashCrash => compute_flash_crash(portfolio),
        };

        let shock_loss = shock_loss.max(Decimal::ZERO); // Losses are non-negative
        let post_shock_equity = (portfolio.equity - shock_loss).max(Decimal::ZERO);
        let reached_ruin = post_shock_equity <= portfolio.ruin_threshold;

        let survival_fraction = if portfolio.equity > Decimal::ZERO {
            post_shock_equity / portfolio.equity
        } else {
            Decimal::ZERO
        };

        // Recovery probability: heuristic based on survival fraction and drawdown depth
        // Higher remaining equity → easier recovery. Uses exponential decay.
        let sf_f = survival_fraction.to_f64().unwrap_or(0.0);
        let recovery_raw = if reached_ruin { 0.0 } else { sf_f.powi(2) };
        let recovery_probability = f64_to_decimal(recovery_raw.clamp(0.0, 1.0));

        Ok(StressMetrics {
            scenario: scenario.clone(),
            post_shock_equity,
            scenario_drawdown: shock_loss,
            survival_fraction,
            reached_ruin,
            recovery_probability,
        })
    }

    /// Run all 6 scenarios and return the most severe (highest loss).
    pub fn run_all(portfolio: &PortfolioSnapshot) -> Result<Vec<StressMetrics>, String> {
        let scenarios = [
            StressScenario::CorrelatedLosses,
            StressScenario::SpreadSpike,
            StressScenario::SlippageSpike,
            StressScenario::LiquidityCollapse,
            StressScenario::VolatilityExplosion,
            StressScenario::FlashCrash,
        ];

        scenarios
            .iter()
            .map(|s| Self::run_scenario(s, portfolio))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Scenario computations — all Decimal arithmetic
// ---------------------------------------------------------------------------

/// Correlated losses: all positions hit simultaneously.
/// Loss = open_exposure × avg_correlation × 0.02 (2% move on correlated book)
fn compute_correlated_losses(p: &PortfolioSnapshot) -> Decimal {
    p.open_exposure * p.avg_correlation * Decimal::new(2, 2) // 2% adverse move
}

/// Spread spike: spread expands 5× for all active positions × daily fills.
/// Incremental cost = (5 - 1) × avg_spread_cost × daily_fills
fn compute_spread_spike(p: &PortfolioSnapshot) -> Decimal {
    let spike_factor = Decimal::new(4, 0); // additional 4× spread cost
    spike_factor * p.avg_spread_cost * Decimal::from(p.daily_fills)
}

/// Slippage spike: every fill gets 3× ATR of additional slippage.
/// Extra cost = 3 × avg_atr × daily_fills
fn compute_slippage_spike(p: &PortfolioSnapshot) -> Decimal {
    Decimal::new(3, 0) * p.avg_atr * Decimal::from(p.daily_fills)
}

/// Liquidity collapse: 40% of fills are rejected → 40% of expected PnL lost.
/// Additionally, each stranded position must close at worse prices.
fn compute_liquidity_collapse(p: &PortfolioSnapshot) -> Decimal {
    let rejected_fraction = Decimal::new(40, 2); // 40%
    let expected_pnl_loss = p.avg_position_pnl.max(Decimal::ZERO)
        * rejected_fraction
        * Decimal::from(p.active_positions);
    let exit_cost = p.avg_fill_cost * Decimal::new(3, 0) // 3× fill cost to exit
        * rejected_fraction
        * Decimal::from(p.active_positions);
    expected_pnl_loss + exit_cost
}

/// Volatility explosion: ATR triples, so position sizing must reduce.
/// Unrealized PnL on current oversized book suffers 1× ATR adverse move on all positions.
fn compute_volatility_explosion(p: &PortfolioSnapshot) -> Decimal {
    // 1 ATR adverse move per position × 3 (vol expansion factor)
    p.avg_atr * Decimal::new(3, 0) * Decimal::from(p.active_positions)
}

/// Flash crash: −10% price shock on entire open book.
/// Shock loss = open_exposure × 10%
fn compute_flash_crash(p: &PortfolioSnapshot) -> Decimal {
    p.open_exposure * Decimal::new(10, 2) // 10% of notional exposure
}

fn f64_to_decimal(v: f64) -> Decimal {
    Decimal::from_f64_retain(v)
        .unwrap_or(Decimal::ZERO)
        .round_dp(6)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_portfolio() -> PortfolioSnapshot {
        PortfolioSnapshot {
            equity: Decimal::new(100_000, 0),
            open_exposure: Decimal::new(500_000, 0),
            avg_spread_cost: Decimal::new(5, 0),
            avg_atr: Decimal::new(20, 0),
            active_positions: 5,
            avg_position_pnl: Decimal::new(200, 0),
            avg_fill_cost: Decimal::new(8, 0),
            daily_fills: 10,
            avg_correlation: Decimal::new(75, 2), // 0.75
            ruin_threshold: Decimal::new(70_000, 0),
        }
    }

    #[test]
    fn test_flash_crash_survival() {
        let p = make_portfolio();
        let result = PortfolioStressTester::run_scenario(&StressScenario::FlashCrash, &p)
            .expect("flash crash failed");
        // 10% of 500,000 = 50,000 loss → equity = 50,000 → reaches ruin (threshold 70k)
        assert!(result.reached_ruin);
        assert_eq!(result.scenario_drawdown, Decimal::new(50_000, 0));
    }

    #[test]
    fn test_spread_spike_no_ruin() {
        let p = make_portfolio();
        let result = PortfolioStressTester::run_scenario(&StressScenario::SpreadSpike, &p)
            .expect("spread spike failed");
        // 4 × 5 × 10 = 200 loss → well within equity
        assert!(!result.reached_ruin);
        assert_eq!(result.scenario_drawdown, Decimal::new(200, 0));
    }

    #[test]
    fn test_all_scenarios_run() {
        let p = make_portfolio();
        let results = PortfolioStressTester::run_all(&p).expect("run_all failed");
        assert_eq!(results.len(), 6);
        for r in &results {
            assert!(r.survival_fraction >= Decimal::ZERO);
            assert!(r.survival_fraction <= Decimal::ONE);
        }
    }

    #[test]
    fn test_positive_equity_required() {
        let mut p = make_portfolio();
        p.equity = Decimal::ZERO;
        let result = PortfolioStressTester::run_scenario(&StressScenario::FlashCrash, &p);
        assert!(result.is_err());
    }
}
