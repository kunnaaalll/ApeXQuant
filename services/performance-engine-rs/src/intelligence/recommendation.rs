#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceAction {
    Increase,
    Maintain,
    Reduce,
    Freeze,
}

#[derive(Debug, Clone)]
pub struct Recommendation {
    pub action: ConfidenceAction,
    pub why: String,
    pub largest_contributor: String,
    pub largest_penalty: String,
    pub what_improved: String,
    pub what_degraded: String,
    pub confidence_path: String,
}

impl Recommendation {
    pub fn new(
        action: ConfidenceAction,
        why: String,
        largest_contributor: String,
        largest_penalty: String,
        what_improved: String,
        what_degraded: String,
        confidence_path: String,
    ) -> Self {
        Self {
            action,
            why,
            largest_contributor,
            largest_penalty,
            what_improved,
            what_degraded,
            confidence_path,
        }
    }
}
