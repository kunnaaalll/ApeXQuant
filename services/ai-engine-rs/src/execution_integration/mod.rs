use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FillQuality {
    pub order_id: Uuid,
    pub price_improvement: Decimal,
    pub fill_ratio: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlippageMetrics {
    pub expected_slippage: Decimal,
    pub actual_slippage: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LatencyMetrics {
    pub routing_latency_ms: u64,
    pub execution_latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionRiskScore {
    pub score: Decimal, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiquidityHealth {
    pub health_score: Decimal, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionConfidence {
    pub confidence_score: Decimal, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrokerHealthScore {
    pub broker_id: String,
    pub health_score: Decimal, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutingRecommendations {
    pub preferred_broker: String,
    pub fallback_broker: String,
}

pub struct ExecutionIntegration;

impl ExecutionIntegration {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate_execution(
        _fill: &FillQuality,
        slippage: &SlippageMetrics,
        latency: &LatencyMetrics,
        risk: &ExecutionRiskScore,
    ) -> (ExecutionConfidence, BrokerHealthScore, RoutingRecommendations) {
        
        let confidence_score = if slippage.actual_slippage > slippage.expected_slippage {
            Decimal::new(50, 2)
        } else {
            Decimal::new(90, 2)
        };

        let health_score = if latency.execution_latency_ms > 100 || risk.score > Decimal::new(80, 2) {
            Decimal::new(40, 2)
        } else {
            Decimal::new(95, 2)
        };

        (
            ExecutionConfidence { confidence_score },
            BrokerHealthScore {
                broker_id: "PRIMARY_BROKER".to_string(),
                health_score,
            },
            RoutingRecommendations {
                preferred_broker: if health_score > Decimal::new(50, 2) { "PRIMARY_BROKER".to_string() } else { "SECONDARY_BROKER".to_string() },
                fallback_broker: "TERTIARY_BROKER".to_string(),
            }
        )
    }
}
