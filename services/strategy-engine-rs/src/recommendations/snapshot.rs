use super::Recommendation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecommendationSnapshot {
    pub recommendation: Recommendation,
}

impl RecommendationSnapshot {
    pub fn new(recommendation: Recommendation) -> Self {
        Self { recommendation }
    }
}
