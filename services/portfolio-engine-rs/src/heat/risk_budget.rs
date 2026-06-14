use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapitalPressureState {
    Low,
    Normal,
    Elevated,
    High,
    Extreme,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapitalPressureAssessment {
    pub state: CapitalPressureState,
    pub open_risk_pressure: Decimal,
    pub margin_pressure: Decimal,
    pub drawdown_pressure: Decimal,
    pub correlation_pressure: Decimal,
    pub volatility_pressure: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskBudget {
    pub max_portfolio_risk: Decimal,
    pub utilized_risk: Decimal,
    pub remaining_risk: Decimal,
    pub reserved_risk: Decimal,
    pub emergency_reserve: Decimal,
    pub total_risk_capacity: Decimal,
}

impl RiskBudget {
    pub fn new(
        max_portfolio_risk: Decimal,
        utilized_risk: Decimal,
        reserved_risk: Decimal,
        emergency_reserve: Decimal,
    ) -> Self {
        let total_risk_capacity = max_portfolio_risk;
        
        let remaining_risk = if utilized_risk + reserved_risk + emergency_reserve >= total_risk_capacity {
            Decimal::ZERO
        } else {
            total_risk_capacity - utilized_risk - reserved_risk - emergency_reserve
        };

        Self {
            max_portfolio_risk,
            utilized_risk,
            remaining_risk,
            reserved_risk,
            emergency_reserve,
            total_risk_capacity,
        }
    }

    pub fn can_allocate(&self, additional_risk: Decimal) -> bool {
        self.remaining_risk >= additional_risk
    }
}
