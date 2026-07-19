use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::circuit_breaker::events::CircuitBreakerEvent;
use crate::correlation::events::CorrelationRiskEvent;
use crate::drawdown::events::DrawdownEvent;
use crate::exposure::events::ExposureRiskEvent;
use crate::recommendations::events::RecommendationEvent;
use crate::stress::events::StressEvent;
use crate::var::events::VarRiskEvent;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventRecord {
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub sequence: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    pub payload: PortfolioEventWrapper,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PortfolioEventWrapper {
    Drawdown(DrawdownEvent),
    Exposure(ExposureRiskEvent),
    Correlation(CorrelationRiskEvent),
    Var(VarRiskEvent),
    CircuitBreaker(CircuitBreakerEvent),
    Recommendation(RecommendationEvent),
    Stress(StressEvent),
}
