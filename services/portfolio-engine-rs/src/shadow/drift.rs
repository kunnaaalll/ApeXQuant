use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriftState {
    Stable,
    Watch,
    Elevated,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftAssessment {
    pub overall_state: DriftState,
    pub metrics: HashMap<String, f64>,
}

pub struct DriftMonitor;

impl Default for DriftMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl DriftMonitor {
    pub fn new() -> Self {
        Self
    }

    /// Assess drift based on current metrics
    pub fn assess(&self, metrics: HashMap<String, f64>) -> DriftAssessment {
        let mut overall_state = DriftState::Stable;

        for (key, value) in &metrics {
            let state = match key.as_str() {
                "health_drift" | "quality_drift" => {
                    if *value > 0.05 {
                        DriftState::Critical
                    } else if *value > 0.02 {
                        DriftState::Elevated
                    } else if *value > 0.01 {
                        DriftState::Watch
                    } else {
                        DriftState::Stable
                    }
                }
                "drawdown_drift" | "heat_drift" => {
                    if *value > 0.02 {
                        DriftState::Critical
                    } else if *value > 0.01 {
                        DriftState::Elevated
                    } else if *value > 0.005 {
                        DriftState::Watch
                    } else {
                        DriftState::Stable
                    }
                }
                _ => DriftState::Stable,
            };

            // Upgrade overall state if needed
            match (overall_state.clone(), state) {
                (DriftState::Stable, new_state) => overall_state = new_state,
                (DriftState::Watch, DriftState::Elevated | DriftState::Critical) => overall_state = DriftState::Elevated, // Needs refined logic
                (DriftState::Elevated, DriftState::Critical) => overall_state = DriftState::Critical,
                _ => {}
            }
        }

        DriftAssessment {
            overall_state,
            metrics,
        }
    }
}
