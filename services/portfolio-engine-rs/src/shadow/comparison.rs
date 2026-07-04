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
        let mut differences = Vec::new();
        Self::compare_json(expected_ts, actual_rust, "", &mut differences);

        if differences.is_empty() {
            return ComparisonResult::new(Classification::ExactMatch, vec![]);
        }

        let mut has_major = false;
        let mut has_minor = false;

        for diff in &differences {
            let exp_f: Result<f64, _> = diff.expected.trim_matches('"').parse();
            let act_f: Result<f64, _> = diff.actual.trim_matches('"').parse();
            if let (Ok(e), Ok(a)) = (exp_f, act_f) {
                let diff_abs = (e - a).abs();
                if diff_abs > 0.05 {
                    has_major = true;
                } else if diff_abs > 0.0001 {
                    has_minor = true;
                }
            } else {
                has_major = true;
            }
        }

        let classification = if has_major {
            Classification::Mismatch
        } else if has_minor {
            Classification::MinorDifference
        } else {
            Classification::CloseMatch
        };

        ComparisonResult::new(classification, differences)
    }

    fn compare_json(expected: &serde_json::Value, actual: &serde_json::Value, path: &str, differences: &mut Vec<DifferenceDetail>) {
        match (expected, actual) {
            (serde_json::Value::Object(exp_map), serde_json::Value::Object(act_map)) => {
                for (key, exp_val) in exp_map {
                    let current_path = if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };
                    if let Some(act_val) = act_map.get(key) {
                        Self::compare_json(exp_val, act_val, &current_path, differences);
                    } else {
                        differences.push(DifferenceDetail {
                            field: current_path,
                            expected: exp_val.to_string(),
                            actual: "null".to_string(),
                            reason: "Missing key in actual value".to_string(),
                        });
                    }
                }
                for (key, act_val) in act_map {
                    if !exp_map.contains_key(key) {
                        let current_path = if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };
                        differences.push(DifferenceDetail {
                            field: current_path,
                            expected: "null".to_string(),
                            actual: act_val.to_string(),
                            reason: "Extra key in actual value".to_string(),
                        });
                    }
                }
            }
            (serde_json::Value::Array(exp_arr), serde_json::Value::Array(act_arr)) => {
                let len = std::cmp::max(exp_arr.len(), act_arr.len());
                for i in 0..len {
                    let current_path = format!("{}[{}]", path, i);
                    match (exp_arr.get(i), act_arr.get(i)) {
                        (Some(exp_val), Some(act_val)) => {
                            Self::compare_json(exp_val, act_val, &current_path, differences);
                        }
                        (Some(exp_val), None) => {
                            differences.push(DifferenceDetail {
                                field: current_path,
                                expected: exp_val.to_string(),
                                actual: "null".to_string(),
                                reason: "Array element missing in actual".to_string(),
                            });
                        }
                        (None, Some(act_val)) => {
                            differences.push(DifferenceDetail {
                                field: current_path,
                                expected: "null".to_string(),
                                actual: act_val.to_string(),
                                reason: "Array element extra in actual".to_string(),
                            });
                        }
                        (None, None) => {}
                    }
                }
            }
            (exp_val, act_val) => {
                if exp_val != act_val {
                    differences.push(DifferenceDetail {
                        field: path.to_string(),
                        expected: exp_val.to_string(),
                        actual: act_val.to_string(),
                        reason: "Value mismatch".to_string(),
                    });
                }
            }
        }
    }
}
