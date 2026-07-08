use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::BTreeMap;

use crate::storage::ClosedTradeRecord;
use crate::drawdown::DrawdownCalculator;
use crate::analytics::engine::AnalyticsEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloResult {
    pub total_trials: u64,
    pub original_expectancy: Decimal,
    pub survival_rate: Decimal,           // Probability of avoiding ruin
    pub collapse_probability: Decimal,    // Probability of hitting risk of ruin
    pub max_drawdown_95: Decimal,         // 95th percentile worst max drawdown
    pub median_drawdown: Decimal,         // Median max drawdown
    pub terminal_equity_95: Decimal,      // 95th percentile best terminal equity
    pub terminal_equity_05: Decimal,      // 5th percentile worst terminal equity
    pub median_confidence: Decimal,       // Derived confidence from simulation stability
}

pub struct PerformanceMonteCarlo {
    seed: u64,
}

impl PerformanceMonteCarlo {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Run pseudo-random but deterministic trials using bootstrap resampling.
    /// Resamples from historical trade R-outcomes to build synthetic equity curves.
    pub fn simulate(
        &self,
        trials: u64,
        historical_trades: &[ClosedTradeRecord],
        ruin_threshold_r: Decimal,
    ) -> MonteCarloResult {
        if historical_trades.is_empty() {
            return MonteCarloResult {
                total_trials: trials,
                original_expectancy: Decimal::ZERO,
                survival_rate: dec!(1.0),
                collapse_probability: Decimal::ZERO,
                max_drawdown_95: Decimal::ZERO,
                median_drawdown: Decimal::ZERO,
                terminal_equity_95: Decimal::ZERO,
                terminal_equity_05: Decimal::ZERO,
                median_confidence: Decimal::ZERO,
            };
        }

        let r_outcomes: Vec<Decimal> = historical_trades.iter().map(|t| t.r_outcome).collect();
        let trade_count = r_outcomes.len();
        
        let mut rng = StdRng::seed_from_u64(self.seed);
        
        let mut ruin_count = 0u64;
        let mut max_drawdowns = Vec::with_capacity(trials as usize);
        let mut terminal_equities = Vec::with_capacity(trials as usize);

        for _ in 0..trials {
            let mut current_equity = Decimal::ZERO;
            let mut equity_curve = Vec::with_capacity(trade_count + 1);
            equity_curve.push(Decimal::ZERO);
            
            let mut ruined = false;

            for _ in 0..trade_count {
                // Uniform sampling with replacement (bootstrap)
                let idx = rng.gen_range(0..trade_count);
                let r = r_outcomes[idx];
                
                current_equity += r;
                equity_curve.push(current_equity);
                
                if current_equity <= -ruin_threshold_r {
                    ruined = true;
                    // We don't break early; we want to capture the full path for stats,
                    // but we do count it as a ruin event.
                }
            }
            
            if ruined {
                ruin_count += 1;
            }
            
            let trial_max_dd = DrawdownCalculator::calculate_max_drawdown(&equity_curve);
            max_drawdowns.push(trial_max_dd);
            terminal_equities.push(current_equity);
        }

        // Sort vectors for percentile calculations
        max_drawdowns.sort_unstable();
        terminal_equities.sort_unstable();

        let survival_rate = Decimal::from(trials - ruin_count) / Decimal::from(trials);
        let collapse_probability = dec!(1.0) - survival_rate;

        let max_drawdown_95 = Self::percentile(&max_drawdowns, 0.95);
        let median_drawdown = Self::percentile(&max_drawdowns, 0.50);
        let terminal_equity_95 = Self::percentile(&terminal_equities, 0.95);
        let terminal_equity_05 = Self::percentile(&terminal_equities, 0.05);

        // Confidence logic based on survival rate and median drawdown
        // 1.0 survival and <20% drawdown gives high confidence
        let confidence_survival = survival_rate;
        let confidence_dd = if median_drawdown <= dec!(0.20) {
            dec!(1.0)
        } else if median_drawdown >= dec!(0.50) {
            Decimal::ZERO
        } else {
            (dec!(0.50) - median_drawdown) / dec!(0.30)
        };
        
        let median_confidence = (confidence_survival * dec!(0.7) + confidence_dd * dec!(0.3))
            .clamp(Decimal::ZERO, dec!(1.0));

        // Original Expectancy via AnalyticsEngine helpers
        let original_expectancy = {
            let win_count = historical_trades.iter().filter(|t| t.is_win).count() as u32;
            let loss_count = historical_trades.iter().filter(|t| !t.is_win && t.pnl_usd < Decimal::ZERO).count() as u32;
            let breakeven = trade_count as u32 - win_count - loss_count;
            let gross_profit: Decimal = historical_trades.iter().filter(|t| t.is_win).map(|t| t.pnl_usd).sum();
            let gross_loss: Decimal = historical_trades.iter().filter(|t| !t.is_win && t.pnl_usd < Decimal::ZERO).map(|t| t.pnl_usd.abs()).sum();
            
            crate::expectancy::calculator::ExpectancyCalculator::calculate(
                win_count, loss_count, breakeven, gross_profit, gross_loss
            ).expectancy
        };

        MonteCarloResult {
            total_trials: trials,
            original_expectancy,
            survival_rate,
            collapse_probability,
            max_drawdown_95,
            median_drawdown,
            terminal_equity_95,
            terminal_equity_05,
            median_confidence,
        }
    }

    fn percentile(sorted: &[Decimal], p: f64) -> Decimal {
        if sorted.is_empty() {
            return Decimal::ZERO;
        }
        let max_idx = sorted.len() - 1;
        let idx = (p * max_idx as f64).round() as usize;
        sorted[idx.clamp(0, max_idx)]
    }
}
