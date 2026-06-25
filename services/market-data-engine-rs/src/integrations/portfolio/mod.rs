use rust_decimal::Decimal;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCorrelationMatrix {
    pub correlations: HashMap<String, HashMap<String, Decimal>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversificationScore {
    pub score: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureBuckets {
    pub allocations: HashMap<String, Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatSignals {
    pub overheating_assets: Vec<String>,
    pub cooling_assets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationSuggestions {
    pub symbol: String,
    pub suggested_weight: Decimal,
    pub rationale: String,
}
