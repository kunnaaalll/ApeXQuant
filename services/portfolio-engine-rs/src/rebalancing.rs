use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceTarget {
    pub symbol: String,
    pub target_weight: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceAction {
    pub symbol: String,
    pub current_weight: Decimal,
    pub target_weight: Decimal,
    pub weight_delta: Decimal,
    pub is_buy: bool,
}

pub struct RebalanceEngine {
    tolerance_threshold: Decimal,
}

impl RebalanceEngine {
    pub fn new(tolerance_threshold: Decimal) -> Self {
        Self {
            tolerance_threshold,
        }
    }

    pub fn calculate_actions(
        &self,
        current_weights: &[(String, Decimal)],
        targets: &[RebalanceTarget],
    ) -> Vec<RebalanceAction> {
        let mut actions = Vec::new();

        for target in targets {
            let current_weight = current_weights
                .iter()
                .find(|(s, _)| s == &target.symbol)
                .map(|(_, w)| *w)
                .unwrap_or(Decimal::ZERO);

            let weight_delta = target.target_weight - current_weight;

            if weight_delta.abs() > self.tolerance_threshold {
                actions.push(RebalanceAction {
                    symbol: target.symbol.clone(),
                    current_weight,
                    target_weight: target.target_weight,
                    weight_delta,
                    is_buy: weight_delta.is_sign_positive(),
                });
            }
        }

        // Handle cases where a current holding is not in targets (should be sold)
        for (symbol, current_weight) in current_weights {
            let is_in_targets = targets.iter().any(|t| &t.symbol == symbol);
            
            if !is_in_targets && !current_weight.is_zero()
                && current_weight.abs() > self.tolerance_threshold {
                    actions.push(RebalanceAction {
                        symbol: symbol.clone(),
                        current_weight: *current_weight,
                        target_weight: Decimal::ZERO,
                        weight_delta: -*current_weight,
                        is_buy: false,
                    });
                }
        }

        actions
    }
}
