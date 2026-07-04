use crate::filters::FilterResult;
use crate::signals::SignalResult;

pub struct SessionFilter;

impl SessionFilter {
    /// Filter out signals that occur during undesirable times or off-hours
    pub fn filter(&self, signal: &SignalResult, active_session: &str) -> FilterResult {
        if active_session == "OffHours" {
            FilterResult::Reject(format!(
                "Rejecting signal {} due to OffHours trading restrictions",
                signal.signal_id
            ))
        } else {
            FilterResult::Pass
        }
    }
}
