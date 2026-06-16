use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismResult {
    pub total_runs: u64,
    pub is_deterministic: bool,
    pub divergence_detected_at: Option<u64>,
}

pub struct DeterminismValidator;

impl DeterminismValidator {
    pub fn new() -> Self {
        Self
    }

    /// Runs a simulation N times and guarantees identical outputs.
    /// In a real implementation, `F` would represent a complex state mutation sequence.
    pub fn validate<F, R>(&self, runs: u64, mut action: F) -> DeterminismResult
    where
        F: FnMut() -> R,
        R: PartialEq + std::fmt::Debug,
    {
        if runs == 0 {
            return DeterminismResult {
                total_runs: 0,
                is_deterministic: true,
                divergence_detected_at: None,
            };
        }

        let baseline = action();

        for i in 1..runs {
            let next_result = action();
            if next_result != baseline {
                return DeterminismResult {
                    total_runs: runs,
                    is_deterministic: false,
                    divergence_detected_at: Some(i),
                };
            }
        }

        DeterminismResult {
            total_runs: runs,
            is_deterministic: true,
            divergence_detected_at: None,
        }
    }
}
