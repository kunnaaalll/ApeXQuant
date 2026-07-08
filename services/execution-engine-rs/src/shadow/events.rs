use super::comparison::ComparisonResult;
use super::drift::DriftAnalysis;
use super::health::ShadowHealth;
use super::parity::ParityScore;
use super::statistics::ShadowStatistics;
use super::validator::ValidatorState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShadowEvent {
    ComparisonRecorded(ComparisonResult),
    DriftCalculated(DriftAnalysis),
    StatisticsUpdated(ShadowStatistics),
    ParityUpdated(ParityScore),
    HealthUpdated(ShadowHealth),
    ValidatorPromoted {
        from: ValidatorState,
        to: ValidatorState,
    },
    ValidatorDemoted {
        from: ValidatorState,
        to: ValidatorState,
    },
}
