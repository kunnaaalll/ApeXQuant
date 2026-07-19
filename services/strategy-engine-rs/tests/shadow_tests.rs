#![allow(warnings, clippy::all, deprecated)]
use rust_decimal::Decimal;
use strategy_engine_rs::shadow::comparison::{ComparisonEngine, ShadowComparisonState};
use strategy_engine_rs::shadow::drift::DriftEngine;
use strategy_engine_rs::shadow::events::ShadowEvent;
use strategy_engine_rs::shadow::reporter::Reporter;
use strategy_engine_rs::shadow::snapshot::ShadowSnapshot;
use strategy_engine_rs::shadow::statistics::StatisticsEngine;
use strategy_engine_rs::shadow::validator::{GoLiveState, GoLiveValidator};

#[test]
fn test_shadow_match_bounds() {
    let engine = ComparisonEngine::new();
    let state = engine.compare(
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
    );
    assert_eq!(state, ShadowComparisonState::ExactMatch);

    let state = engine.compare(
        Decimal::new(1, 2),
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
    );
    assert_eq!(state, ShadowComparisonState::CloseMatch);
}

#[test]
fn test_drift_percentage_bounds() {
    let drift = DriftEngine::new();
    let val = drift.measure_relative_difference(Decimal::new(200, 0), Decimal::ZERO, Decimal::ONE);
    // Absolute diff is 200. Max ref is 1. Pct is 20000. Clamped to 100.
    assert_eq!(val, Decimal::new(100, 0));
}

#[test]
fn test_match_percentage_bounds() {
    let mut stats = StatisticsEngine::new();
    assert_eq!(stats.match_percentage(), Decimal::ZERO);

    stats.record(ShadowComparisonState::ExactMatch);
    assert_eq!(stats.match_percentage(), Decimal::new(100, 0));

    stats.record(ShadowComparisonState::Mismatch);
    assert_eq!(stats.match_percentage(), Decimal::new(50, 0));
}

#[test]
fn test_validator_progression() {
    let mut validator = GoLiveValidator::new();
    assert_eq!(validator.state, GoLiveState::NotReady);

    for _ in 0..100 {
        validator.process(ShadowComparisonState::ExactMatch);
    }
    assert_eq!(validator.state, GoLiveState::Monitoring);

    for _ in 0..900 {
        validator.process(ShadowComparisonState::ExactMatch);
    }
    assert_eq!(validator.state, GoLiveState::Candidate);

    for _ in 0..9000 {
        validator.process(ShadowComparisonState::ExactMatch);
    }
    assert_eq!(validator.state, GoLiveState::Approved);
}

#[test]
fn test_forbidden_transitions() {
    let mut validator = GoLiveValidator::new();

    // get to approved
    for _ in 0..10000 {
        validator.process(ShadowComparisonState::ExactMatch);
    }
    assert_eq!(validator.state, GoLiveState::Approved);

    // failure drops to candidate
    validator.process(ShadowComparisonState::Warning);
    assert_eq!(validator.state, GoLiveState::Candidate);
    assert_eq!(validator.consecutive_exact_matches, 999);

    // failure drops to monitoring
    validator.process(ShadowComparisonState::Warning);
    assert_eq!(validator.state, GoLiveState::Monitoring);
    assert_eq!(validator.consecutive_exact_matches, 99);

    // failure drops to not ready
    validator.process(ShadowComparisonState::Warning);
    assert_eq!(validator.state, GoLiveState::NotReady);
    assert_eq!(validator.consecutive_exact_matches, 0);
}

#[test]
fn test_event_rebuild() {
    let event = ShadowEvent::ComparisonProcessed {
        total_difference: Decimal::new(15, 1),
    };
    let clone = event.clone();
    assert_eq!(event, clone);
}

#[test]
fn test_snapshot_restore() {
    let snapshot = ShadowSnapshot {
        match_percentage: Decimal::new(95, 0),
        consecutive_exact_matches: 1000,
        go_live_state: GoLiveState::Candidate,
    };
    let clone = snapshot.clone();
    assert_eq!(snapshot, clone);
}

#[test]
fn test_report_generation() {
    let mut stats = StatisticsEngine::new();
    stats.record(ShadowComparisonState::ExactMatch);
    stats.record(ShadowComparisonState::CloseMatch);

    let reporter = Reporter::new();
    let markdown = reporter.generate_markdown_report(&stats);
    assert!(markdown.contains("Exact Matches: 1"));
    assert!(markdown.contains("Close Matches: 1"));

    let json = reporter.generate_json_report(&stats);
    assert!(json.contains("\"exact_matches\": 1"));
    assert!(json.contains("\"close_matches\": 1"));
}

#[test]
fn test_determinism_100k_iterations() {
    let engine = ComparisonEngine::new();
    let mut stats = StatisticsEngine::new();
    let mut validator = GoLiveValidator::new();

    for _ in 0..100_000 {
        let state = engine.compare(
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
        );
        stats.record(state);
        validator.process(state);
    }

    assert_eq!(stats.match_percentage(), Decimal::new(100, 0));
    assert_eq!(validator.state, GoLiveState::Approved);
}
