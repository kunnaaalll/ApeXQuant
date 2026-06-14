use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Classification {
    ExactMatch,
    CloseMatch,
    MinorDifference,
    MajorDifference,
    Mismatch,
    MissingData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferenceDetail {
    pub field: String,
    pub expected: String,
    pub actual: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub classification: Classification,
    pub differences: Vec<DifferenceDetail>,
    pub metadata: serde_json::Value,
}

impl ComparisonResult {
    pub fn new(classification: Classification, differences: Vec<DifferenceDetail>) -> Self {
        Self {
            classification,
            differences,
            metadata: serde_json::Value::Null,
        }
    }
}

/// The main comparison engine for shadow mode.
pub struct PortfolioComparison;

impl PortfolioComparison {
    pub fn compare(expected_ts: &serde_json::Value, actual_rust: &serde_json::Value) -> ComparisonResult {
        // A dummy implementation for initial observation
        if expected_ts == actual_rust {
            return ComparisonResult::new(Classification::ExactMatch, vec![]);
        }

        ComparisonResult::new(
            Classification::Mismatch,
            vec![DifferenceDetail {
                field: "root".to_string(),
                expected: expected_ts.to_string(),
                actual: actual_rust.to_string(),
                reason: "Values differ".to_string(),
            }],
        )
    }
}
