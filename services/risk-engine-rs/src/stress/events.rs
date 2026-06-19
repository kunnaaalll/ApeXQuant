use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use super::scenarios::HistoricalScenario;
use super::survival::SurvivalState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StressTestExecuted {
    pub scenario: HistoricalScenario,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioTriggered {
    pub scenario: HistoricalScenario,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SurvivalStateChanged {
    pub previous_state: SurvivalState,
    pub new_state: SurvivalState,
    pub score: Decimal,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StressEvent {
    TestExecuted(StressTestExecuted),
    ScenarioTriggered(ScenarioTriggered),
    StateChanged(SurvivalStateChanged),
}
