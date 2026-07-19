use rust_decimal::Decimal;

use crate::allocation::capital_allocator::CapitalAllocator;
use crate::allocation::models::TradeAdmissionDecision;
use crate::allocation::recovery::AllocationRecoveryModel;
use crate::allocation::reserve_manager::{OpportunityReserveAssessment, ReserveManager};
use crate::exposure::global::GlobalExposure;
use crate::heat::heat_score::{HeatContributionBreakdown, PortfolioHeat};
use crate::heat::risk_budget::RiskBudget;

#[test]
fn test_reserve_manager_initialization() -> Result<(), Box<dyn std::error::Error>> {
    let rm = ReserveManager::new(Decimal::new(10000, 0), Decimal::new(5000, 0))?;
    assert_eq!(rm.total_reserved(), Decimal::new(15000, 0));
    Ok(())
}

#[test]
fn test_reserve_manager_negative_fails() {
    let result = ReserveManager::new(Decimal::new(-100, 0), Decimal::new(5000, 0));
    assert!(result.is_err());
}

#[test]
fn test_recovery_model_decay() {
    let mut rm = AllocationRecoveryModel::new(Decimal::new(10, 0), Decimal::new(10, 2)); // 10.0, 0.10
    rm.update_drawdown(Decimal::new(12, 0));
    assert!(rm.is_in_drawdown);
    assert_eq!(rm.recovery_progress, Decimal::new(0, 0));

    // Recovery starts when drawdown goes below threshold
    rm.update_drawdown(Decimal::new(8, 0));
    rm.tick_decay();
    assert_eq!(rm.recovery_progress, Decimal::new(10, 2));

    for _ in 0..10 {
        rm.tick_decay();
    }
    assert_eq!(rm.recovery_progress, Decimal::new(1, 0));
    assert!(!rm.is_in_drawdown);
}

#[test]
fn test_trade_admission_rejected_on_heat() -> Result<(), Box<dyn std::error::Error>> {
    let rm = ReserveManager::new(Decimal::new(1000, 0), Decimal::new(1000, 0))?;
    let rec_model = AllocationRecoveryModel::new(Decimal::new(10, 0), Decimal::new(5, 2));
    let mut allocator = CapitalAllocator::new(rm, rec_model);

    // Heat is 90 (Critical)
    let heat = PortfolioHeat::new(
        90,
        HeatContributionBreakdown {
            factors: vec![],
            total_score: 90,
        },
    );
    let risk_budget = RiskBudget::new(
        Decimal::new(10000, 0),
        Decimal::new(5000, 0),
        Decimal::new(0, 0),
        Decimal::new(0, 0),
    );
    let global_exposure = GlobalExposure::new();

    let decision = allocator.evaluate_trade_admission(
        &heat,
        &risk_budget,
        Decimal::new(1000, 0),
        Decimal::new(500, 0),
        &global_exposure,
        false,
        0,
    )?;

    assert!(!decision.can_accept_trade);
    assert_eq!(decision.admission_decision, TradeAdmissionDecision::Reject);
    assert_eq!(decision.allocation_size, Decimal::new(0, 0));
    Ok(())
}

#[test]
fn test_trade_admission_reduced_on_heat() -> Result<(), Box<dyn std::error::Error>> {
    let rm = ReserveManager::new(Decimal::new(1000, 0), Decimal::new(1000, 0))?;
    let rec_model = AllocationRecoveryModel::new(Decimal::new(10, 0), Decimal::new(5, 2));
    let mut allocator = CapitalAllocator::new(rm, rec_model);

    // Heat is 70 (Hot)
    let heat = PortfolioHeat::new(
        70,
        HeatContributionBreakdown {
            factors: vec![],
            total_score: 70,
        },
    );
    let risk_budget = RiskBudget::new(
        Decimal::new(10000, 0),
        Decimal::new(5000, 0),
        Decimal::new(0, 0),
        Decimal::new(0, 0),
    );
    let global_exposure = GlobalExposure::new();

    let decision = allocator.evaluate_trade_admission(
        &heat,
        &risk_budget,
        Decimal::new(1000, 0),
        Decimal::new(500, 0),
        &global_exposure,
        false,
        0,
    )?;

    assert!(decision.can_accept_trade);
    assert_eq!(
        decision.admission_decision,
        TradeAdmissionDecision::ApproveReduced
    );
    assert_eq!(decision.allocation_size, Decimal::new(500, 0)); // 50% reduction
    Ok(())
}

#[test]
fn test_trade_admission_rejected_on_risk_budget() -> Result<(), Box<dyn std::error::Error>> {
    let rm = ReserveManager::new(Decimal::new(1000, 0), Decimal::new(1000, 0))?;
    let rec_model = AllocationRecoveryModel::new(Decimal::new(10, 0), Decimal::new(5, 2));
    let mut allocator = CapitalAllocator::new(rm, rec_model);

    // Heat is 20 (Cold)
    let heat = PortfolioHeat::new(
        20,
        HeatContributionBreakdown {
            factors: vec![],
            total_score: 20,
        },
    );

    // Risk budget exceeded
    let risk_budget = RiskBudget::new(
        Decimal::new(10000, 0),
        Decimal::new(9000, 0),
        Decimal::new(0, 0),
        Decimal::new(0, 0),
    );
    let global_exposure = GlobalExposure::new();

    // Asking for 2000 risk, but only 1000 available
    let decision = allocator.evaluate_trade_admission(
        &heat,
        &risk_budget,
        Decimal::new(2000, 0),
        Decimal::new(2000, 0),
        &global_exposure,
        false,
        0,
    )?;

    assert!(!decision.can_accept_trade);
    assert_eq!(decision.admission_decision, TradeAdmissionDecision::Reject);
    Ok(())
}

#[test]
fn test_trade_admission_opportunity_reserve() -> Result<(), Box<dyn std::error::Error>> {
    let mut rm = ReserveManager::new(Decimal::new(1000, 0), Decimal::new(1000, 0))?;

    // Give 500 in opportunity reserve
    rm.update_opportunity_reserve(OpportunityReserveAssessment {
        is_exceptional_opportunity: true,
        required_reserve: Decimal::new(500, 0),
        confidence: Decimal::new(1, 0),
        reason: "Exceptional".into(),
    })?;

    let rec_model = AllocationRecoveryModel::new(Decimal::new(10, 0), Decimal::new(5, 2));
    let mut allocator = CapitalAllocator::new(rm, rec_model);

    let heat = PortfolioHeat::new(
        20,
        HeatContributionBreakdown {
            factors: vec![],
            total_score: 20,
        },
    );
    let risk_budget = RiskBudget::new(
        Decimal::new(2600, 0),
        Decimal::new(0, 0),
        Decimal::new(0, 0),
        Decimal::new(0, 0),
    ); // total available 2600. Reserves = 2500
    let global_exposure = GlobalExposure::new();

    // Deploy 400. Total available 2600, reserves 2500. Without opportunity reserve, 400 would push it below 2500 reserves.
    // But since is_opportunity=true, we can use the opportunity reserve!
    let decision = allocator.evaluate_trade_admission(
        &heat,
        &risk_budget,
        Decimal::new(400, 0),
        Decimal::new(100, 0),
        &global_exposure,
        true,
        0,
    )?;

    assert!(decision.can_accept_trade);
    assert_eq!(decision.admission_decision, TradeAdmissionDecision::Approve);
    assert!(decision
        .contributing_factors
        .iter()
        .any(|f| f.name == "Opportunity Reserve Used"));
    Ok(())
}
