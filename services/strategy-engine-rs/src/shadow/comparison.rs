use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowComparisonState {
    ExactMatch,
    CloseMatch,
    Warning,
    Mismatch,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ComparisonEngine;

impl Default for ComparisonEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ComparisonEngine {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare(
        &self,
        strategy_health: Decimal,
        confidence: Decimal,
        edge: Decimal,
        drift: Decimal,
        allocation: Decimal,
        degradation: Decimal,
        context_ranking: Decimal,
        recommendation: Decimal,
        cluster_profile: Decimal,
        meta_intelligence: Decimal,
        reference_health: Decimal,
        reference_confidence: Decimal,
        reference_edge: Decimal,
        reference_drift: Decimal,
        reference_allocation: Decimal,
        reference_degradation: Decimal,
        reference_context_ranking: Decimal,
        reference_recommendation: Decimal,
        reference_cluster_profile: Decimal,
        reference_meta_intelligence: Decimal,
    ) -> ShadowComparisonState {
        let h_diff = (strategy_health - reference_health).abs();
        let c_diff = (confidence - reference_confidence).abs();
        let e_diff = (edge - reference_edge).abs();
        let d_diff = (drift - reference_drift).abs();
        let a_diff = (allocation - reference_allocation).abs();
        let deg_diff = (degradation - reference_degradation).abs();
        let cr_diff = (context_ranking - reference_context_ranking).abs();
        let rec_diff = (recommendation - reference_recommendation).abs();
        let cp_diff = (cluster_profile - reference_cluster_profile).abs();
        let mi_diff = (meta_intelligence - reference_meta_intelligence).abs();

        let total_diff = h_diff
            + c_diff
            + e_diff
            + d_diff
            + a_diff
            + deg_diff
            + cr_diff
            + rec_diff
            + cp_diff
            + mi_diff;

        if total_diff == Decimal::ZERO {
            ShadowComparisonState::ExactMatch
        } else if total_diff <= Decimal::new(1, 2) {
            ShadowComparisonState::CloseMatch
        } else if total_diff <= Decimal::new(5, 2) {
            ShadowComparisonState::Warning
        } else if total_diff <= Decimal::new(20, 2) {
            ShadowComparisonState::Mismatch
        } else {
            ShadowComparisonState::Critical
        }
    }
}
