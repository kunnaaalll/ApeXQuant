use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllocationState {
    pub strategy_allocations: Vec<(Uuid, Decimal)>,
    pub unallocated_capital: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorrelationClusters {
    pub cluster_id: Uuid,
    pub correlated_strategies: Vec<Uuid>,
    pub cluster_exposure: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapitalEfficiency {
    pub roe: Decimal, // Return on equity
    pub capital_utilization: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountHealth {
    pub margin_health: Decimal,
    pub equity_curve_stability: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PayoutState {
    pub pending_payouts: Decimal,
    pub available_for_reinvestment: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllocationChanges {
    pub strategy_id: Uuid,
    pub delta: Decimal, // Positive or negative
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapitalRotationSuggestions {
    pub from_strategy_id: Uuid,
    pub to_strategy_id: Uuid,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountScalingRecommendations {
    pub suggested_total_capital: Decimal,
}

pub struct PortfolioIntegration;

impl PortfolioIntegration {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate_portfolio(
        _alloc: &AllocationState,
        _clusters: &[CorrelationClusters],
        efficiency: &CapitalEfficiency,
        health: &AccountHealth,
    ) -> (Vec<AllocationChanges>, Vec<CapitalRotationSuggestions>, AccountScalingRecommendations) {
        
        let mut scaling_rec = AccountScalingRecommendations {
            suggested_total_capital: Decimal::new(0, 0)
        };

        if efficiency.capital_utilization > Decimal::new(80, 2) && health.margin_health > Decimal::new(90, 2) {
            // Placeholder: Recommend scaling up account
            scaling_rec.suggested_total_capital = Decimal::new(2000000, 0); 
        }

        (
            vec![], // No generic changes generated in isolation
            vec![],
            scaling_rec
        )
    }
}
