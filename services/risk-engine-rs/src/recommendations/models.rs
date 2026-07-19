#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum RecommendationStrength {
    Frozen,
    Restricted,
    Conservative,
    Moderate,
    Aggressive,
    VeryAggressive,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum RiskRecommendation {
    FreezeTrading,
    EmergencyReduction,
    ReduceAggressively,
    ReduceRisk,
    MaintainRisk,
    IncreaseRisk,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum TradeAdmissionPolicy {
    Freeze,
    Block,
    Delay,
    Allow,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecommendationExplanation {
    pub why: String,
    pub what_improved: String,
    pub what_deteriorated: String,
    pub dominant_factor: String,
    pub prevented_stronger_recommendation: String,
}

// Input states based on the spec
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DrawdownState {
    Healthy,
    Warning,
    Collapse,
    Frozen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExposureState {
    Healthy,
    Warning,
    Collapse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CorrelationSeverity {
    Healthy,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VarSeverity {
    Healthy,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CircuitBreakerState {
    Healthy,
    Restricted,
    Frozen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TailRiskScore {
    Healthy,
    Warning,
    Collapse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HiddenLeverage {
    Healthy,
    Warning,
    Collapse,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RiskInputs {
    pub drawdown_state: DrawdownState,
    pub exposure_state: ExposureState,
    pub correlation_severity: CorrelationSeverity,
    pub var_severity: VarSeverity,
    pub circuit_breaker_state: CircuitBreakerState,
    pub tail_risk_score: TailRiskScore,
    pub hidden_leverage: HiddenLeverage,
    pub exposure_concentration: ExposureState, // Assume same enum for simplicity or a bool? Spec says: "Exposure concentration"
}

// Result of an individual engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IncreaseDecision {
    Reject,
    Delay,
    Maintain,
    Increase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReduceDecision {
    EmergencyReduction,
    ReduceAggressively,
    ReduceModerately,
    ReduceSlightly,
    NoAction,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RiskCommitteeDecision {
    pub recommendation: RiskRecommendation,
    pub admission_policy: TradeAdmissionPolicy,
    pub explanation: RecommendationExplanation,
    pub confidence: u32, // e.g. 0 to 100
    pub timestamp: u64,
}
