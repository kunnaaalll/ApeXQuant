use uuid::Uuid;

pub trait ShadowValidator {
    fn validate_recommendation(
        &self,
        ai_recommendation_id: Uuid,
        human_decision_id: Option<Uuid>,
        actual_outcome_id: Option<Uuid>,
    ) -> Result<crate::shadow_validation::ShadowMetrics, &'static str>;
}
