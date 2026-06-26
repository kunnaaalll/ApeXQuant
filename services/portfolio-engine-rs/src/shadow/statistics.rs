use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowStatistics {
    pub agreement_percentage: Decimal,
    pub exact_match_percentage: Decimal,
    pub close_match_percentage: Decimal,
    pub major_mismatch_percentage: Decimal,
    pub average_drift: Decimal,
    pub max_drift: Decimal,
    // Add time windows (1h, 1d, 1w, 1m) inside specific aggregations
    pub window: WindowPeriod,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WindowPeriod {
    OneHour,
    OneDay,
    OneWeek,
    OneMonth,
}

pub struct StatisticsEngine;

impl Default for StatisticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StatisticsEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn aggregate(&self, _events: &[crate::shadow::ShadowEvent], window: WindowPeriod) -> ShadowStatistics {
        // Implementation will aggregate statistics over the period
        ShadowStatistics {
            agreement_percentage: Decimal::new(100, 0),
            exact_match_percentage: Decimal::new(100, 0),
            close_match_percentage: Decimal::ZERO,
            major_mismatch_percentage: Decimal::ZERO,
            average_drift: Decimal::ZERO,
            max_drift: Decimal::ZERO,
            window,
        }
    }
}
