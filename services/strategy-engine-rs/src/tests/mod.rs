use rust_decimal::Decimal;
use crate::state::StrategyState;
use crate::health::HealthScore;
use crate::confidence::ConfidenceScore;
use crate::degradation::{DegradationEngine, DegradationState};
use crate::recovery::{RecoveryEngine, RecoveryState};
// use crate::ranking::{StrategyRank, RankTier};

#[test]
fn test_state_transitions() {
    let state = StrategyState::Dormant;
    assert!(!state.can_transition_to(&StrategyState::Strong));
    assert!(state.can_transition_to(&StrategyState::Research));

    let retired = StrategyState::Retired;
    assert!(!retired.can_transition_to(&StrategyState::Active));

    let paused = StrategyState::Paused;
    assert!(!paused.can_transition_to(&StrategyState::Elite));
}

#[test]
fn test_score_bounds() {
    let health = HealthScore::new(Decimal::from(150));
    assert_eq!(health.value(), Decimal::from(100));

    let health_low = HealthScore::new(Decimal::from(-50));
    assert_eq!(health_low.value(), Decimal::from(0));

    let conf = ConfidenceScore::new(Decimal::from(200));
    assert_eq!(conf.value(), Decimal::from(100));

    let conf_low = ConfidenceScore::new(Decimal::from(-10));
    assert_eq!(conf_low.value(), Decimal::from(0));
}

#[test]
fn test_degradation_thresholds() {
    let engine = DegradationEngine::new();
    assert_eq!(engine.evaluate(Decimal::from(10)), DegradationState::Healthy);
    assert_eq!(engine.evaluate(Decimal::from(-10)), DegradationState::EarlyWarning);
    assert_eq!(engine.evaluate(Decimal::from(-25)), DegradationState::Weakening);
    assert_eq!(engine.evaluate(Decimal::from(-45)), DegradationState::Danger);
    assert_eq!(engine.evaluate(Decimal::from(-65)), DegradationState::Collapse);
}

#[test]
fn test_recovery_logic() {
    let mut engine = RecoveryEngine::new();
    assert_eq!(engine.state(), RecoveryState::None);

    engine.record_cycle(true);
    engine.record_cycle(true);
    assert_eq!(engine.state(), RecoveryState::Slow);

    engine.record_cycle(false);
    assert_eq!(engine.state(), RecoveryState::None);

    for _ in 0..20 {
        engine.record_cycle(true);
    }
    assert_eq!(engine.state(), RecoveryState::Exceptional);
}

#[test]
fn test_determinism() {
    // 100,000 iterations without divergence or precision drift
    let mut base_health = Decimal::from(100);
    let decay = Decimal::from(1) / Decimal::from(1000);
    
    for _ in 0..100_000 {
        base_health -= decay;
    }
    
    assert_eq!(base_health, Decimal::from(0));
}

mod context_tests;
mod intelligence_tests;
mod confidence_tests;
mod streaks_tests;
mod drift_tests;
mod learning_tests;
mod memory_tests;
mod determinism_tests;
