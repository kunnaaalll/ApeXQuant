use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversificationMetrics {
    pub herfindahl_hirschman_index: Decimal, // HHI index for concentration
    pub max_sector_weight: Decimal,
    pub max_asset_weight: Decimal,
}

pub struct DiversificationEngine;

impl Default for DiversificationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DiversificationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_metrics(
        &self,
        asset_weights: &[(String, Decimal)],
        sector_weights: &HashMap<String, Decimal>,
    ) -> DiversificationMetrics {
        // HHI = sum of squares of weights
        let mut hhi = Decimal::ZERO;
        let mut max_asset = Decimal::ZERO;

        for (_, weight) in asset_weights {
            let abs_w = weight.abs();
            hhi += abs_w * abs_w;
            if abs_w > max_asset {
                max_asset = abs_w;
            }
        }

        let max_sector = sector_weights
            .values()
            .map(|w| w.abs())
            .max()
            .unwrap_or(Decimal::ZERO);

        DiversificationMetrics {
            herfindahl_hirschman_index: hhi,
            max_sector_weight: max_sector,
            max_asset_weight: max_asset,
        }
    }
}
