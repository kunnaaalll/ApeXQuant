use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOutcome {
    pub trade_id: Uuid,
    pub strategy_id: String,
    pub profit_loss: Decimal,
    pub max_drawdown: Decimal,
    pub execution_slippage: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyWeightUpdate {
    pub strategy_id: String,
    pub new_weight: Decimal,
    pub performance_delta: Decimal,
}

pub struct ReinforcementEngine;

impl ReinforcementEngine {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn process_outcome(
        &self,
        outcome: &TradeOutcome,
        current_weight: Decimal,
    ) -> StrategyWeightUpdate {
        let mut new_weight = current_weight;
        // Deterministic reward function
        if outcome.profit_loss > Decimal::ZERO {
            // Reward: increase weight based on profit
            let reward = outcome.profit_loss / Decimal::new(1000, 0); // Normalized reward
            let safe_reward = if reward > Decimal::new(5, 2) {
                Decimal::new(5, 2)
            } else {
                reward
            }; // Cap at 0.05
            new_weight += safe_reward;
        } else {
            // Penalty: decrease weight based on loss
            let penalty = (outcome.profit_loss.abs() / Decimal::new(500, 0))
                + (outcome.max_drawdown / Decimal::new(100, 0));
            let safe_penalty = if penalty > Decimal::new(1, 1) {
                Decimal::new(1, 1)
            } else {
                penalty
            }; // Cap at 0.1
            new_weight -= safe_penalty;
        }

        // Cap weight bounds deterministically
        if new_weight < Decimal::ZERO {
            new_weight = Decimal::ZERO;
        } else if new_weight > Decimal::ONE {
            new_weight = Decimal::ONE;
        }

        StrategyWeightUpdate {
            strategy_id: outcome.strategy_id.clone(),
            new_weight,
            performance_delta: new_weight - current_weight,
        }
    }
}
