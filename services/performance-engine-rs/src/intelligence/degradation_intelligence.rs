use crate::intelligence::edge_intelligence::{EdgeIntelligence, EdgeState};
use crate::intelligence::expectancy_intelligence::{ExpectancyAssessment, ExpectancyState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradationState {
    Healthy,
    Monitor,
    Degrading,
    Critical,
}

#[derive(Debug, Clone)]
pub struct DegradationIntelligence {
    pub state: DegradationState,
    pub warning_triggered: bool,
    pub reason: String,
}

impl DegradationIntelligence {
    pub fn evaluate(edge: &EdgeIntelligence, expectancy: &ExpectancyAssessment) -> Self {
        let mut warning_triggered = false;
        let mut state = DegradationState::Healthy;
        let mut reason = String::from("System operating within expected parameters");

        if edge.edge_state == EdgeState::Critical || expectancy.state == ExpectancyState::Negative {
            state = DegradationState::Critical;
            warning_triggered = true;
            reason = String::from("Critical degradation detected in edge or expectancy");
        } else if edge.edge_state == EdgeState::Weakening
            || expectancy.state == ExpectancyState::Weak
        {
            state = DegradationState::Degrading;
            warning_triggered = true;
            reason = String::from(
                "Performance is degrading. Recent metrics trailing long-term averages",
            );
        } else if edge.degrading || expectancy.negative_drift {
            state = DegradationState::Monitor;
            reason = String::from("Early signs of negative drift. Monitor closely");
        }

        Self {
            state,
            warning_triggered,
            reason,
        }
    }
}
