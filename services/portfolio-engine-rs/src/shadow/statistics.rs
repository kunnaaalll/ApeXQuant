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

use std::str::FromStr;
use rust_decimal::prelude::FromPrimitive;

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

    pub fn aggregate(&self, events: &[crate::shadow::ShadowEvent], window: WindowPeriod) -> ShadowStatistics {
        let comparisons: Vec<_> = events.iter()
            .filter(|e| e.event_type == crate::shadow::ShadowEventType::ComparisonPerformed)
            .collect();

        if comparisons.is_empty() {
            return ShadowStatistics {
                agreement_percentage: Decimal::new(100, 0),
                exact_match_percentage: Decimal::new(100, 0),
                close_match_percentage: Decimal::ZERO,
                major_mismatch_percentage: Decimal::ZERO,
                average_drift: Decimal::ZERO,
                max_drift: Decimal::ZERO,
                window,
            };
        }

        let total = comparisons.len() as i64;
        let mut exact_matches = 0;
        let mut close_matches = 0;
        let mut major_mismatches = 0;
        
        let mut total_drift = Decimal::ZERO;
        let mut max_drift = Decimal::ZERO;
        let mut drift_count = 0;

        for event in &comparisons {
            if let Some(classification_str) = event.details.get("classification").and_then(|v| v.as_str()) {
                match classification_str {
                    "ExactMatch" => exact_matches += 1,
                    "CloseMatch" => close_matches += 1,
                    "MajorDifference" | "Mismatch" => major_mismatches += 1,
                    _ => {}
                }
            }

            if let Some(drift_val) = event.details.get("average_drift").and_then(|v| v.as_str()) {
                if let Ok(d) = Decimal::from_str(drift_val) {
                    total_drift += d;
                    if d > max_drift {
                        max_drift = d;
                    }
                    drift_count += 1;
                }
            } else if let Some(drift_val) = event.details.get("average_drift").and_then(|v| v.as_f64()) {
                if let Some(d) = Decimal::from_f64(drift_val) {
                    total_drift += d;
                    if d > max_drift {
                        max_drift = d;
                    }
                    drift_count += 1;
                }
            }
        }

        let exact_match_percentage = Decimal::new(exact_matches * 100, 0) / Decimal::new(total, 0);
        let close_match_percentage = Decimal::new(close_matches * 100, 0) / Decimal::new(total, 0);
        let major_mismatch_percentage = Decimal::new(major_mismatches * 100, 0) / Decimal::new(total, 0);
        let agreement_percentage = Decimal::new((exact_matches + close_matches) * 100, 0) / Decimal::new(total, 0);

        let average_drift = if drift_count > 0 {
            total_drift / Decimal::new(drift_count, 0)
        } else {
            Decimal::ZERO
        };

        ShadowStatistics {
            agreement_percentage,
            exact_match_percentage,
            close_match_percentage,
            major_mismatch_percentage,
            average_drift,
            max_drift,
            window,
        }
    }
}
