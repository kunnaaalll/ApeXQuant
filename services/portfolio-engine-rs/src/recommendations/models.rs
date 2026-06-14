use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecommendationExplanation {
    pub why: String,
    pub what_changed: String,
    pub what_improved: Option<String>,
    pub what_deteriorated: Option<String>,
    pub most_contributing_factor: String,
    pub prevented_stronger_recommendation: Option<String>,
}

impl RecommendationExplanation {
    pub fn new(
        why: impl Into<String>,
        what_changed: impl Into<String>,
        most_contributing_factor: impl Into<String>,
    ) -> Self {
        Self {
            why: why.into(),
            what_changed: what_changed.into(),
            what_improved: None,
            what_deteriorated: None,
            most_contributing_factor: most_contributing_factor.into(),
            prevented_stronger_recommendation: None,
        }
    }

    pub fn with_improvements(mut self, improved: impl Into<String>) -> Self {
        self.what_improved = Some(improved.into());
        self
    }

    pub fn with_deteriorations(mut self, deteriorated: impl Into<String>) -> Self {
        self.what_deteriorated = Some(deteriorated.into());
        self
    }

    pub fn with_prevented(mut self, prevented: impl Into<String>) -> Self {
        self.prevented_stronger_recommendation = Some(prevented.into());
        self
    }
}
