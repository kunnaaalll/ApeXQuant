use crate::filters::FilterResult;
use crate::signals::SignalResult;

pub struct RegimeFilter;

impl RegimeFilter {
    /// Filter out signals that don't match the current market regime
    pub fn filter(&self, signal: &SignalResult, regime_trending: bool) -> FilterResult {
        // If trending, reject counter-trend signals or signal directions that aren't trending
        if regime_trending && signal.confluence_score < 70.0 {
            FilterResult::Reject(format!(
                "Rejecting signal {} due to low trend confluence score ({:.1}) during trending regime",
                signal.signal_id, signal.confluence_score
            ))
        } else {
            FilterResult::Pass
        }
    }
}
