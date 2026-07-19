use super::Recommendation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecommendationEvent {
    RecommendationGenerated { recommendation: Recommendation },
}
