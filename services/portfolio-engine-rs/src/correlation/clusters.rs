use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClusterState {
    Low,
    Normal,
    Elevated,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClusterType {
    UsdConcentration,
    RiskOn,
    Index,
    Commodity,
    Crypto,
    Sector(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationCluster {
    pub cluster_type: ClusterType,
    pub state: ClusterState,
    pub constituent_symbols: Vec<String>,
    pub internal_correlation: Decimal,
    pub weight_in_portfolio: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioCorrelationResult {
    pub clusters: Vec<CorrelationCluster>,
    pub hidden_leverage_detected: bool,
    pub duplicate_themes: Vec<String>,
    pub overexposure_warnings: Vec<String>,
    pub cross_symbol_concentration: HashMap<String, Decimal>,
}
