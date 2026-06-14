use rust_decimal::Decimal;

use crate::heat::heat_score::{PortfolioHeat, PortfolioHeatState};
use crate::heat::risk_budget::RiskBudget;
use crate::exposure::global::GlobalExposure;

use super::models::{
    AdmissionFactor, AllocationState, CapitalAllocationDecision,
    TradeAdmissionDecision,
};
use super::reserve_manager::ReserveManager;
use super::recovery::AllocationRecoveryModel;
use super::errors::AllocationError;

#[derive(Debug, Clone)]
pub struct CapitalAllocator {
    pub reserve_manager: ReserveManager,
    pub recovery_model: AllocationRecoveryModel,
    pub current_state: AllocationState,
    pub base_confidence: Decimal,
}

impl CapitalAllocator {
    pub fn new(
        reserve_manager: ReserveManager,
        recovery_model: AllocationRecoveryModel,
    ) -> Self {
        Self {
            reserve_manager,
            recovery_model,
            current_state: AllocationState::Normal,
            base_confidence: Decimal::ONE,
        }
    }

    pub fn evaluate_trade_admission(
        &mut self,
        heat: &PortfolioHeat,
        risk_budget: &RiskBudget,
        requested_capital: Decimal,
        requested_risk: Decimal,
        global_exposure: &GlobalExposure,
        is_opportunity: bool,
        timestamp: i64,
    ) -> Result<CapitalAllocationDecision, AllocationError> {
        let mut factors = Vec::new();
        
        // 1. Evaluate Heat
        let heat_factor = match heat.state {
            PortfolioHeatState::Frozen => {
                factors.push(AdmissionFactor {
                    name: "Heat Frozen".into(),
                    description: "Portfolio heat is at maximum threshold".into(),
                    blocks_admission: true,
                });
                TradeAdmissionDecision::Freeze
            }
            PortfolioHeatState::Critical => {
                factors.push(AdmissionFactor {
                    name: "Heat Critical".into(),
                    description: "Portfolio is severely overheated".into(),
                    blocks_admission: true,
                });
                TradeAdmissionDecision::Reject
            }
            PortfolioHeatState::Hot => {
                factors.push(AdmissionFactor {
                    name: "Heat Hot".into(),
                    description: "Portfolio is hot, admitting reduced sizes".into(),
                    blocks_admission: false,
                });
                TradeAdmissionDecision::ApproveReduced
            }
            _ => TradeAdmissionDecision::Approve,
        };

        // 2. Evaluate Risk Budget
        let risk_factor = if !risk_budget.can_allocate(requested_risk) {
            factors.push(AdmissionFactor {
                name: "Risk Budget Exceeded".into(),
                description: "Not enough remaining risk budget".into(),
                blocks_admission: true,
            });
            TradeAdmissionDecision::Reject
        } else {
            TradeAdmissionDecision::Approve
        };

        // 3. Evaluate Reserves
        let total_available = risk_budget.total_risk_capacity; // Using risk capacity as capital proxy for now
        let mut reserve_decision = TradeAdmissionDecision::Approve;

        if !self.reserve_manager.can_deploy(requested_capital, total_available) {
            if is_opportunity && requested_capital <= self.reserve_manager.opportunity_reserve {
                factors.push(AdmissionFactor {
                    name: "Opportunity Reserve Used".into(),
                    description: "Deploying from opportunity reserve".into(),
                    blocks_admission: false,
                });
            } else {
                factors.push(AdmissionFactor {
                    name: "Reserve Limit Hit".into(),
                    description: "Cannot deploy without breaching minimum reserves".into(),
                    blocks_admission: true,
                });
                reserve_decision = TradeAdmissionDecision::Reject;
            }
        }

        // 4. Combine Decisions
        let mut final_decision = TradeAdmissionDecision::Approve;
        for decision in [&heat_factor, &risk_factor, &reserve_decision] {
            final_decision = match (final_decision, decision) {
                (TradeAdmissionDecision::Freeze, _) | (_, TradeAdmissionDecision::Freeze) => TradeAdmissionDecision::Freeze,
                (TradeAdmissionDecision::Reject, _) | (_, TradeAdmissionDecision::Reject) => TradeAdmissionDecision::Reject,
                (TradeAdmissionDecision::Delay, _) | (_, TradeAdmissionDecision::Delay) => TradeAdmissionDecision::Delay,
                (TradeAdmissionDecision::ApproveReduced, _) | (_, TradeAdmissionDecision::ApproveReduced) => TradeAdmissionDecision::ApproveReduced,
                (TradeAdmissionDecision::Approve, TradeAdmissionDecision::Approve) => TradeAdmissionDecision::Approve,
            };
        }

        // Modify allocation size based on final decision
        let mut final_allocation_size = requested_capital;
        if final_decision == TradeAdmissionDecision::ApproveReduced {
            // Apply a 50% reduction
            final_allocation_size = final_allocation_size * Decimal::new(50, 2);
        } else if matches!(final_decision, TradeAdmissionDecision::Reject | TradeAdmissionDecision::Freeze | TradeAdmissionDecision::Delay) {
            final_allocation_size = Decimal::ZERO;
        }

        let is_accepted = final_allocation_size > Decimal::ZERO && !matches!(final_decision, TradeAdmissionDecision::Reject | TradeAdmissionDecision::Freeze | TradeAdmissionDecision::Delay);

        Ok(CapitalAllocationDecision {
            can_accept_trade: is_accepted,
            admission_decision: final_decision,
            allocation_size: final_allocation_size,
            remaining_capacity: risk_budget.remaining_risk,
            reserved_capacity: self.reserve_manager.total_reserved(),
            emergency_capacity: self.reserve_manager.emergency_reserve,
            reason: if is_accepted { "Trade admitted".into() } else { "Trade rejected by admission factors".into() },
            contributing_factors: factors,
            heat_contribution: heat.score,
            exposure_contribution: global_exposure.net_exposure,
            confidence: self.base_confidence,
            timestamp,
        })
    }
}
