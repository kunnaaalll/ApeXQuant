use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StrategyPerformance {
    pub strategy_id: Uuid,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
    pub sharpe_ratio: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StrategyConfidence {
    pub strategy_id: Uuid,
    pub confidence_score: Decimal, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StrategyLifecycle {
    Incubation,
    Testing,
    Production,
    Sunsetting,
    Retired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StrategyDrift {
    pub strategy_id: Uuid,
    pub drift_detected: bool,
    pub drift_severity: Decimal, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromotionRecommendations {
    pub strategy_id: Uuid,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StrategyRanking {
    pub strategy_id: Uuid,
    pub rank: u32,
    pub total_score: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StrategyScalingRecommendation {
    ScaleUp,
    Maintain,
    ScaleDown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StrategyRetirementRecommendation {
    pub strategy_id: Uuid,
    pub retire: bool,
    pub reason: String,
}

pub struct StrategyIntegration;

impl StrategyIntegration {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate_strategy(
        perf: &StrategyPerformance,
        conf: &StrategyConfidence,
        drift: &StrategyDrift,
        _lifecycle: &StrategyLifecycle,
    ) -> (StrategyRanking, StrategyScalingRecommendation, StrategyRetirementRecommendation) {
        let total_score = perf.win_rate * conf.confidence_score;
        let ranking = StrategyRanking {
            strategy_id: perf.strategy_id,
            rank: 1, // Placeholder for logic combining multiple
            total_score,
        };

        let scaling = if total_score > Decimal::new(75, 2) && !drift.drift_detected {
            StrategyScalingRecommendation::ScaleUp
        } else if drift.drift_severity > Decimal::new(50, 2) {
            StrategyScalingRecommendation::ScaleDown
        } else {
            StrategyScalingRecommendation::Maintain
        };

        let retirement = StrategyRetirementRecommendation {
            strategy_id: perf.strategy_id,
            retire: drift.drift_severity > Decimal::new(80, 2),
            reason: if drift.drift_severity > Decimal::new(80, 2) {
                "Severe drift detected".to_string()
            } else {
                "".to_string()
            },
        };

        (ranking, scaling, retirement)
    }
}
