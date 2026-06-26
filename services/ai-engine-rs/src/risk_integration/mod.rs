use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortfolioHeat {
    pub heat_score: Decimal, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaR {
    pub value_at_risk_95: Decimal,
    pub value_at_risk_99: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DrawdownMetrics {
    pub current_drawdown: Decimal,
    pub max_drawdown_limit: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExposureMetrics {
    pub gross_exposure: Decimal,
    pub net_exposure: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CircuitBreakerState {
    Normal,
    Warning,
    Tripped,
    CoolingDown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskApprovalScore {
    pub score: Decimal, // 0.0 to 1.0 (Recommendations only, Risk Engine enforces)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllocationLimits {
    pub max_allocation_per_strategy: Decimal,
    pub max_total_allocation: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScalingPermissions {
    pub can_scale_up: bool,
    pub can_scale_down: bool,
}

pub struct RiskIntegration;

impl RiskIntegration {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate_risk(
        heat: &PortfolioHeat,
        _var: &VaR,
        drawdown: &DrawdownMetrics,
        circuit_breaker: &CircuitBreakerState,
    ) -> (RiskApprovalScore, AllocationLimits, ScalingPermissions) {
        
        // AI never overrides risk decisions.
        let mut score = Decimal::new(100, 2);
        let mut can_scale = true;

        if *circuit_breaker == CircuitBreakerState::Tripped || *circuit_breaker == CircuitBreakerState::CoolingDown {
            score = Decimal::new(0, 0);
            can_scale = false;
        }

        if heat.heat_score > Decimal::new(80, 2) || drawdown.current_drawdown >= drawdown.max_drawdown_limit {
            score = Decimal::new(20, 2);
            can_scale = false;
        }

        (
            RiskApprovalScore { score },
            AllocationLimits {
                max_allocation_per_strategy: Decimal::new(100000, 0), // Placeholder
                max_total_allocation: Decimal::new(1000000, 0),
            },
            ScalingPermissions {
                can_scale_up: can_scale,
                can_scale_down: true, // usually allowed to scale down
            }
        )
    }
}
