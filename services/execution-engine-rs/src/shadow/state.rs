use super::drift::DriftAnalysis;
use super::health::ShadowHealth;
use super::parity::ParityScore;
use super::statistics::ShadowStatistics;
use super::validator::GoLiveValidator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowState {
    pub statistics: ShadowStatistics,
    pub parity_score: Option<ParityScore>,
    pub drift_score: Option<DriftAnalysis>,
    pub health: Option<ShadowHealth>,
    pub validator: GoLiveValidator,
}

impl Default for ShadowState {
    fn default() -> Self {
        Self::new()
    }
}

impl ShadowState {
    pub const fn new() -> Self {
        Self {
            statistics: ShadowStatistics::new(),
            parity_score: None,
            drift_score: None,
            health: None,
            validator: GoLiveValidator::new(),
        }
    }
}
