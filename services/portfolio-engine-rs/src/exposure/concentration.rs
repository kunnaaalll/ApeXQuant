use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConcentrationAssessment {
    Low,
    Normal,
    Elevated,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DuplicateExposureResult {
    pub description: String,
    pub assessment: ConcentrationAssessment,
    pub symbols_involved: Vec<String>,
}

impl DuplicateExposureResult {
    pub fn new(description: String, assessment: ConcentrationAssessment, symbols_involved: Vec<String>) -> Self {
        Self {
            description,
            assessment,
            symbols_involved,
        }
    }
}
