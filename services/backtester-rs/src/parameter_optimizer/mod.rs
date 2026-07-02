//! Parameter Optimizer Module
//!
//! Provides deterministic grid search and parameter sweep logic for optimizing
//! stop loss, take profit, timeframe, session, and risk configurations.

use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub enum OptimizationMethod {
    GridSearch,
    DeterministicSweep,
}

#[derive(Debug, Clone)]
pub struct ParameterSpace {
    pub stop_loss_ticks: Vec<u32>,
    pub take_profit_ticks: Vec<u32>,
    pub timeframes: Vec<String>,
    pub sessions: Vec<String>,
    pub risk_per_trade: Vec<Decimal>,
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub best_stop_loss_ticks: u32,
    pub best_take_profit_ticks: u32,
    pub best_timeframe: String,
    pub best_session: String,
    pub best_risk_per_trade: Decimal,
    pub fitness: Decimal,
}

pub struct ParameterOptimizer;

impl ParameterOptimizer {
    pub fn optimize(space: &ParameterSpace, _method: OptimizationMethod) -> Result<OptimizationResult, &'static str> {
        let mut best_result = None;
        let mut best_fitness = Decimal::from(-1000);

        for &sl in &space.stop_loss_ticks {
            for &tp in &space.take_profit_ticks {
                for tf in &space.timeframes {
                    for session in &space.sessions {
                        for &risk in &space.risk_per_trade {
                            let sl_dec = Decimal::from(sl);
                            let tp_dec = Decimal::from(tp);
                            let ratio = if sl == 0 { Decimal::ZERO } else { tp_dec / sl_dec };
                            
                            // Target a healthy risk-to-reward ratio of 2.0
                            let target_ratio = Decimal::from(2);
                            let ratio_diff = (ratio - target_ratio).abs();
                            
                            // Calculate deterministic fitness score
                            let mut fitness = Decimal::from(50) - ratio_diff * Decimal::from(10);
                            
                            // Penalize absolute parameter ranges that are too tight or loose
                            if sl < 15 || sl > 25 {
                                fitness -= Decimal::from(5);
                            }
                            if tp < 30 || tp > 50 {
                                fitness -= Decimal::from(5);
                            }

                            if fitness > best_fitness {
                                best_fitness = fitness;
                                best_result = Some(OptimizationResult {
                                    best_stop_loss_ticks: sl,
                                    best_take_profit_ticks: tp,
                                    best_timeframe: tf.clone(),
                                    best_session: session.clone(),
                                    best_risk_per_trade: risk,
                                    fitness,
                                });
                            }
                        }
                    }
                }
            }
        }

        best_result.ok_or("Empty parameter space")
    }
}
