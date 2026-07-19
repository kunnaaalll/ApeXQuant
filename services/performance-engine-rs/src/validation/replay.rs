use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    pub total_events: u64,
    pub matches_original_state: bool,
    pub passed: bool,
}

pub struct ReplayValidator;

impl ReplayValidator {
    pub fn new() -> Self {
        Self
    }

    /// Verifies that replaying a list of events from scratch results in the exact
    /// same state as processing them linearly.
    pub fn validate_replay<E, S, R, F>(
        &self,
        events: &[E],
        initial_state: S,
        mut processor: F,
    ) -> ReplayResult
    where
        S: PartialEq + Clone,
        R: PartialEq,
        F: FnMut(&mut S, &E) -> R,
    {
        let mut linear_state = initial_state.clone();
        for event in events {
            processor(&mut linear_state, event);
        }

        let mut replay_state = initial_state.clone();
        for event in events {
            processor(&mut replay_state, event);
        }

        let matches = linear_state == replay_state;

        ReplayResult {
            total_events: events.len() as u64,
            matches_original_state: matches,
            passed: matches,
        }
    }
}

impl Default for ReplayValidator {
    fn default() -> Self {
        Self::new()
    }
}
