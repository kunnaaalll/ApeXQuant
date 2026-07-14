//! Certification Module
//!
//! Manages the institutional promotion lifecycle of strategies.

use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub enum CertificationStage {
    Research,
    BacktestApproved,
    ShadowApproved,
    Candidate,
    Production,
    Certified,
}

#[derive(Debug, Clone)]
pub struct PromotionCriteria {
    pub min_parity_threshold: Decimal,
    pub min_robustness_threshold: Decimal,
    pub min_trades: u64,
    pub min_months: u32,
    pub max_drift: Decimal,
}

#[derive(Debug, Clone)]
pub struct StrategyMetrics {
    pub current_parity: Decimal,
    pub current_robustness: Decimal,
    pub total_trades: u64,
    pub active_months: u32,
    pub current_drift: Decimal,
}

pub struct CertificationEngine {
    pub criteria: PromotionCriteria,
}

impl CertificationEngine {
    pub fn new(criteria: PromotionCriteria) -> Self {
        Self { criteria }
    }

    pub fn evaluate_promotion(
        &self,
        current_stage: CertificationStage,
        metrics: &StrategyMetrics,
    ) -> (bool, CertificationStage) {
        let is_eligible = metrics.current_parity >= self.criteria.min_parity_threshold
            && metrics.current_robustness >= self.criteria.min_robustness_threshold
            && metrics.total_trades >= self.criteria.min_trades
            && metrics.active_months >= self.criteria.min_months
            && metrics.current_drift <= self.criteria.max_drift;

        if !is_eligible {
            return (false, current_stage);
        }

        let next_stage = match current_stage {
            CertificationStage::Research => CertificationStage::BacktestApproved,
            CertificationStage::BacktestApproved => CertificationStage::ShadowApproved,
            CertificationStage::ShadowApproved => CertificationStage::Candidate,
            CertificationStage::Candidate => CertificationStage::Production,
            CertificationStage::Production => CertificationStage::Certified,
            CertificationStage::Certified => CertificationStage::Certified,
        };

        (true, next_stage)
    }
}
