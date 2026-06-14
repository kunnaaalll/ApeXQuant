// src/analytics/timeframe.rs
use serde::{Deserialize, Serialize};
use super::regime::RegimePerformanceMetrics;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TimeframePerformanceProfile {
    pub m15: RegimePerformanceMetrics,
    pub m30: RegimePerformanceMetrics,
    pub h1: RegimePerformanceMetrics,
    pub h4: RegimePerformanceMetrics,
}

impl TimeframePerformanceProfile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Determines the best performing timeframe by expectancy
    pub fn best_performing_timeframe(&self) -> &'static str {
        let mut best_name = "None";
        let mut best_expectancy = f64::NEG_INFINITY;

        let mut check = |name: &'static str, metrics: &RegimePerformanceMetrics| {
            if metrics.total_trades > 30 && metrics.expectancy > best_expectancy {
                best_expectancy = metrics.expectancy;
                best_name = name;
            }
        };

        check("M15", &self.m15);
        check("M30", &self.m30);
        check("H1", &self.h1);
        check("H4", &self.h4);

        best_name
    }
}
