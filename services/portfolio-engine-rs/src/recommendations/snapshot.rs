use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::block::TradeAdmissionPolicy;
use super::close::CloseAssessment;
use super::increase::IncreaseExposureRecommendation;
use super::reduce::ReductionAssessment;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecommendationSnapshot {
    pub timestamp: OffsetDateTime,
    pub version: u64,
    pub increase_recommendation: IncreaseExposureRecommendation,
    pub reduce_recommendation: ReductionAssessment,
    pub close_recommendation: CloseAssessment,
    pub block_recommendation: TradeAdmissionPolicy,
}

impl RecommendationSnapshot {
    pub fn new(
        version: u64,
        increase_recommendation: IncreaseExposureRecommendation,
        reduce_recommendation: ReductionAssessment,
        close_recommendation: CloseAssessment,
        block_recommendation: TradeAdmissionPolicy,
    ) -> Self {
        Self {
            timestamp: OffsetDateTime::now_utc(),
            version,
            increase_recommendation,
            reduce_recommendation,
            close_recommendation,
            block_recommendation,
        }
    }
}
