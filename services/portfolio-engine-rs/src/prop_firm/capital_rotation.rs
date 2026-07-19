use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationSignal {
    pub from_asset: String,
    pub to_asset: String,
    pub amount: Decimal,
}

pub struct CapitalRotationEngine;

impl Default for CapitalRotationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CapitalRotationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_signals(
        &self,
        current_allocations: &HashMap<String, Decimal>,
        momentum_scores: &HashMap<String, Decimal>,
        rotation_threshold: Decimal,
    ) -> Vec<RotationSignal> {
        let mut signals = Vec::new();

        // Simple momentum-based rotation strategy
        // Find highest momentum asset and lowest momentum asset that we currently hold

        let mut highest_momentum_asset = None;
        let mut highest_momentum = Decimal::ZERO;

        for (asset, score) in momentum_scores {
            if *score > highest_momentum {
                highest_momentum = *score;
                highest_momentum_asset = Some(asset.clone());
            }
        }

        let mut lowest_momentum_held_asset = None;
        let mut lowest_momentum = Decimal::MAX;

        for (asset, allocation) in current_allocations {
            if *allocation > Decimal::ZERO {
                let score = momentum_scores.get(asset).copied().unwrap_or(Decimal::ZERO);
                if score < lowest_momentum {
                    lowest_momentum = score;
                    lowest_momentum_held_asset = Some(asset.clone());
                }
            }
        }

        if let (Some(from), Some(to)) = (lowest_momentum_held_asset, highest_momentum_asset) {
            if highest_momentum - lowest_momentum > rotation_threshold && from != to {
                let amount = current_allocations
                    .get(&from)
                    .copied()
                    .unwrap_or(Decimal::ZERO)
                    * Decimal::new(5, 1); // Rotate 50%
                signals.push(RotationSignal {
                    from_asset: from,
                    to_asset: to,
                    amount,
                });
            }
        }

        signals
    }
}
