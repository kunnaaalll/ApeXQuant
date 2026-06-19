use rust_decimal::Decimal;
use strategy_engine_rs::shadow::comparison::{ComparisonEngine, ShadowComparisonState};
use strategy_engine_rs::shadow::drift::DriftEngine;
use strategy_engine_rs::shadow::statistics::StatisticsEngine;
use strategy_engine_rs::shadow::validator::{GoLiveValidator, GoLiveState};

#[test]
fn test_shadow_match_bounds() {
    let engine = ComparisonEngine::new();
    let state = engine.compare(
        Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO,
        Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO,
        Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO,
        Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO,
    );
    assert_eq!(state, ShadowComparisonState::ExactMatch);

    let state = engine.compare(
        Decimal::new(1, 2), Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO,
        Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO,
        Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO,
        Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO,
    );
    assert_eq!(state, ShadowComparisonState::CloseMatch);
}

#[test]
fn test_match_percentage_clamping() {
    let mut stats = StatisticsEngine::new();
    assert_eq!(stats.match_percentage(), Decimal::ZERO);

    stats.record(ShadowComparisonState::ExactMatch);
    assert_eq!(stats.match_percentage(), Decimal::new(100, 0));

    stats.record(ShadowComparisonState::Mismatch);
    assert_eq!(stats.match_percentage(), Decimal::new(50, 0));
}

#[test]
fn test_go_live_transitions() {
    let mut validator = GoLiveValidator::new();
    assert_eq!(validator.state, GoLiveState::NotReady);

    for _ in 0..100 {
        validator.process(ShadowComparisonState::ExactMatch);
    }
    assert_eq!(validator.state, GoLiveState::Monitoring);

    validator.process(ShadowComparisonState::Warning);
    assert_eq!(validator.state, GoLiveState::NotReady);
}

#[test]
fn test_drift_percentage_clamping() {
    let drift = DriftEngine::new();
    let val = drift.measure_relative_difference(Decimal::new(200, 0), Decimal::ZERO, Decimal::ONE);
    // Absolute diff is 200. Max ref is 1. Pct is 20000. Clamped to 100.
    assert_eq!(val, Decimal::new(100, 0));
}
