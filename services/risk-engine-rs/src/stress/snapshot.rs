use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use super::scenarios::HistoricalScenario;
use super::severity::Severity;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StressSnapshot {
    pub scenario: HistoricalScenario,
    pub severity: Severity,
    pub survival_score: Decimal,
    pub timestamp: i64,
    pub version: u32,
}
