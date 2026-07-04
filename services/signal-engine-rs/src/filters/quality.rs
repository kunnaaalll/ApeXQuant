use crate::filters::FilterResult;
use crate::signals::SignalResult;

pub struct QualityFilter {
    pub min_quality_score: f64,
}

impl QualityFilter {
    pub fn new(min_quality_score: f64) -> Self {
        Self { min_quality_score }
    }

    /// Filter signal based on confluence score or other quality metrics
    pub fn filter(&self, signal: &SignalResult) -> FilterResult {
        if signal.confluence_score < self.min_quality_score {
            FilterResult::Reject(format!(
                "Rejecting signal {} due to low quality score: {:.1} < {:.1}",
                signal.signal_id, signal.confluence_score, self.min_quality_score
            ))
        } else {
            FilterResult::Pass
        }
    }
}
