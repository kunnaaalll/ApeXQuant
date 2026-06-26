use backtester::portfolio_simulation::{PortfolioState, PortfolioSimulator, PortfolioSnapshot};
use backtester::multi_account::{AccountType, AccountGroup, AccountAllocator, AccountHealth};
use backtester::prop_constraints::{PropConstraints, ConstraintEvaluator, AccountState};
use backtester::payout_simulation::{PayoutEligibility, ProfitSplit, PayoutSimulator};
use backtester::capital_rotation::{CapitalFlow, CapitalRotator};
use backtester::account_allocator::{AllocationModel, Allocator};
use backtester::correlation_simulation::{CorrelationSimulator, ConcentrationScore, DiversificationScore};
use backtester::portfolio_stress::{StressScenario, PortfolioStressTester};
use backtester::funded_account_manager::{FundedAccountState, ChallengeStage, FundedManager};
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn test_portfolio_simulation() {
    let result = PortfolioSimulator::simulate(&[]);
    assert!(result.is_ok());
    
    let metrics = result.unwrap();
    assert_eq!(metrics.average_heat, Decimal::ZERO);
}

#[test]
fn test_multi_account_allocation() {
    let group = AccountGroup {
        group_id: "G1".to_string(),
        account_ids: vec!["A1".to_string(), "A2".to_string()],
    };
    let alloc = AccountAllocator::allocate(&group, Decimal::from_str("100000").unwrap());
    assert!(alloc.is_ok());
}

#[test]
fn test_prop_constraints() {
    let constraints = PropConstraints {
        max_daily_drawdown: Decimal::from_str("0.05").unwrap(),
        max_total_drawdown: Decimal::from_str("0.10").unwrap(),
        profit_target: Some(Decimal::from_str("0.08").unwrap()),
        min_trading_days: 5,
        consistency_rule_pct: Some(Decimal::from_str("0.30").unwrap()),
    };
    
    let state = ConstraintEvaluator::evaluate(
        &constraints,
        Decimal::from_str("102000").unwrap(), // Current
        Decimal::from_str("100000").unwrap(), // Starting
        Decimal::from_str("102000").unwrap(), // Highest
        6,            // Trading days
    );
    assert_eq!(state.unwrap(), AccountState::Active);
}

#[test]
fn test_payout_simulation() {
    let split = ProfitSplit {
        trader_pct: Decimal::from_str("0.9").unwrap(),
        firm_pct: Decimal::from_str("0.1").unwrap(),
    };
    let payout = PayoutSimulator::simulate_payout(
        Decimal::from_str("110000").unwrap(), 
        Decimal::from_str("100000").unwrap(), 
        &split
    );
    assert!(payout.is_ok());
}

#[test]
fn test_capital_rotation() {
    let rotation = CapitalRotator::rotate(Decimal::from_str("5000").unwrap(), "FUNDED_1", "PERSONAL_1");
    assert!(rotation.is_ok());
    
    let flow = rotation.unwrap();
    assert_eq!(flow.amount, Decimal::from_str("5000").unwrap());
    assert_eq!(flow.source_account_id, "FUNDED_1");
    assert_eq!(flow.target_account_id, "PERSONAL_1");
}

#[test]
fn test_account_allocator() {
    let account_ids = vec!["A1".to_string(), "A2".to_string()];
    let alloc = Allocator::allocate(
        &AllocationModel::EqualWeight, 
        Decimal::from_str("100000").unwrap(), 
        &account_ids
    );
    assert!(alloc.is_ok());
}

#[test]
fn test_correlation_simulation() {
    let symbols = vec![Decimal::from_str("0.5").unwrap(), Decimal::from_str("0.5").unwrap()];
    let strategies = vec![Decimal::from_str("1.0").unwrap()];
    let accounts = vec![Decimal::from_str("1.0").unwrap()];
    
    let (conc, div) = CorrelationSimulator::simulate(&symbols, &strategies, &accounts).unwrap();
    assert_eq!(conc.total_score, Decimal::ZERO);
    assert_eq!(div.score, Decimal::ZERO);
}

#[test]
fn test_portfolio_stress() {
    let metrics = PortfolioStressTester::run_scenario(
        &StressScenario::CorrelatedFailures,
        Decimal::from_str("100000").unwrap(),
    );
    assert!(metrics.is_ok());
}

#[test]
fn test_funded_account_manager() {
    let mut state = FundedAccountState {
        account_id: "F1".to_string(),
        stage: ChallengeStage::Phase1,
        pass_probability: Decimal::ZERO,
        estimated_days_to_complete: 0,
        expected_payout_timeline_days: 0,
        payout_history: vec![],
    };
    
    let update = FundedManager::update_progress(
        &mut state, 
        Decimal::from_str("105000").unwrap(), 
        Decimal::from_str("0.6").unwrap()
    );
    assert!(update.is_ok());
}
