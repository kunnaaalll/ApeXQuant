use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAgeAssessment {
    pub position_id: Uuid,
    pub age: Duration,
    pub expected_holding_duration: Duration,
    pub is_stale: bool,
    pub is_overextended: bool,
}

pub struct TradeAgingEngine;

impl TradeAgingEngine {
    pub fn evaluate(
        position_id: Uuid,
        age: Duration,
        expected_duration: Duration,
        has_momentum: bool,
    ) -> TradeAgeAssessment {
        // A trade is considered stale if it has exceeded expected duration without momentum
        let is_stale = age > expected_duration && !has_momentum;

        // A trade is overextended if it is held significantly longer than expected (e.g., 2x)
        let is_overextended = age.as_secs() > (expected_duration.as_secs() * 2);

        TradeAgeAssessment {
            position_id,
            age,
            expected_holding_duration: expected_duration,
            is_stale,
            is_overextended,
        }
    }
}
