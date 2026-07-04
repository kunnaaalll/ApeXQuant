use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub validation_id: String,
    pub passed: bool,
    pub errors: Vec<String>,
    pub timestamp: SystemTime,
}

impl ValidationReport {
    pub fn new(validation_id: String) -> Self {
        Self {
            validation_id,
            passed: true,
            errors: Vec::new(),
            timestamp: SystemTime::now(),
        }
    }

    pub fn add_error(&mut self, err: String) {
        self.passed = false;
        self.errors.push(err);
    }
}
