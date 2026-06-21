use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub accepted: bool,
    pub rejected: bool,
    pub reason: Option<String>,
    pub timestamp: i64,
}

impl ExecutionResult {
    pub fn accepted(timestamp: i64) -> Self {
        Self {
            accepted: true,
            rejected: false,
            reason: None,
            timestamp,
        }
    }

    pub fn rejected(reason: String, timestamp: i64) -> Self {
        Self {
            accepted: false,
            rejected: true,
            reason: Some(reason),
            timestamp,
        }
    }
}
