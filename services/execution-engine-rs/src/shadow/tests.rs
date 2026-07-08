use super::comparison::{ComparisonEngine, ComparisonState};
use super::drift::{DriftEngine, DriftState};
use super::events::ShadowEvent;
use super::health::{HealthEngine, ShadowHealth};
use super::parity::{ParityEngine, ParityLevel};
use super::reporter::ReporterEngine;
use super::snapshots::ShadowSnapshot;
use super::state::ShadowState;
use super::statistics::ShadowStatistics;
use super::thresholds::ParityThresholds;
use super::validator::{GoLiveValidator, ValidatorState};
use rust_decimal_macros::dec;

#[test]
fn test_shadow_match_bounds() {
    let engine = ComparisonEngine::new();
    let thresholds = ParityThresholds::institutional();

    // Exact Match
    assert_eq!(
        engine.compare(dec!(100.0), dec!(100.0), &thresholds),
        ComparisonState::ExactMatch
    );
    assert_eq!(
        engine.compare(dec!(100.0), dec!(101.0), &thresholds),
        ComparisonState::ExactMatch
    );

    // Close Match
    assert_eq!(
        engine.compare(dec!(100.0), dec!(102.0), &thresholds),
        ComparisonState::CloseMatch
    );

    // Warning
    assert_eq!(
        engine.compare(dec!(100.0), dec!(104.0), &thresholds),
        ComparisonState::Warning
    );

    // Mismatch
    assert_eq!(
        engine.compare(dec!(100.0), dec!(109.0), &thresholds),
        ComparisonState::Mismatch
    );

    // Critical Mismatch
    assert_eq!(
        engine.compare(dec!(100.0), dec!(111.0), &thresholds),
        ComparisonState::CriticalMismatch
    );
}

#[test]
fn test_drift_percentage_bounds() {
    // 0 -> 100% no overflow
    let drift1 = DriftEngine::calculate(dec!(100.0), dec!(90.0));
    assert_eq!(drift1.relative_drift, dec!(10.0));
    assert_eq!(drift1.state, DriftState::High);

    let drift2 = DriftEngine::calculate(dec!(100.0), dec!(0.0));
    assert_eq!(drift2.relative_drift, dec!(100.0));
    assert_eq!(drift2.state, DriftState::Extreme);

    let drift3 = DriftEngine::calculate(dec!(0.0), dec!(0.0));
    assert_eq!(drift3.relative_drift, dec!(0.0));
    assert_eq!(drift3.state, DriftState::None);

    let drift4 = DriftEngine::calculate(dec!(0.0), dec!(50.0));
    assert_eq!(drift4.relative_drift, dec!(100.0)); // Div by zero fallback clamped to 100
    assert_eq!(drift4.state, DriftState::Extreme);
}

#[test]
fn test_match_percentage_bounds() {
    let mut stats = ShadowStatistics::new();
    stats.exact_match_count = 10;
    stats.critical_mismatch_count = 10;
    assert_eq!(stats.match_percentage(), dec!(50.0));

    let stats_zero = ShadowStatistics::new();
    assert_eq!(stats_zero.match_percentage(), dec!(100.0));
}

#[test]
fn test_validator_progression() {
    let mut validator = GoLiveValidator::new();
    assert_eq!(validator.state, ValidatorState::NotReady);

    for _ in 0..10 {
        validator.process_parity_pass();
    }
    assert_eq!(validator.state, ValidatorState::Monitoring);

    for _ in 0..40 {
        validator.process_parity_pass();
    }
    assert_eq!(validator.state, ValidatorState::Candidate);

    for _ in 0..50 {
        validator.process_parity_pass();
    }
    assert_eq!(validator.state, ValidatorState::Approved);
}

#[test]
fn test_forbidden_transitions() {
    let mut validator = GoLiveValidator::new();
    validator.state = ValidatorState::Approved;

    // Fail once
    validator.process_parity_failure();
    assert_eq!(validator.state, ValidatorState::Candidate);

    // Fail twice
    validator.process_parity_failure();
    assert_eq!(validator.state, ValidatorState::Monitoring);

    // Fail thrice
    validator.process_parity_failure();
    assert_eq!(validator.state, ValidatorState::NotReady);

    // Cannot drop below NotReady
    validator.process_parity_failure();
    assert_eq!(validator.state, ValidatorState::NotReady);
}

#[test]
fn test_event_rebuild() {
    let mut snapshot = ShadowSnapshot::new();

    let mut stats = ShadowStatistics::new();
    stats.exact_match_count = 50;

    snapshot.apply_event(ShadowEvent::StatisticsUpdated(stats.clone()));
    snapshot.apply_event(ShadowEvent::ValidatorPromoted {
        from: ValidatorState::NotReady,
        to: ValidatorState::Monitoring,
    });

    let state = snapshot.rebuild_state();
    assert_eq!(state.statistics.exact_match_count, 50);
    assert_eq!(state.validator.state, ValidatorState::Monitoring);
}

#[test]
fn test_snapshot_restore() {
    let mut snapshot = ShadowSnapshot::new();
    let drift = DriftEngine::calculate(dec!(100.0), dec!(99.0));

    snapshot.apply_event(ShadowEvent::DriftCalculated(drift.clone()));
    let state = snapshot.rebuild_state();

    assert_eq!(state.drift_score, Some(drift));
}

#[test]
fn test_report_generation() {
    let mut state = ShadowState::new();
    state.statistics.exact_match_count = 42;

    let md = ReporterEngine::generate_markdown(&state);
    assert!(md.contains("Exact Matches: 42"));

    let json = ReporterEngine::generate_json(&state);
    assert!(json.contains("\"exact_matches\": 42"));
}

#[test]
fn test_parity_score_bounds() {
    let mut stats = ShadowStatistics::new();
    stats.exact_match_count = 100;

    let score1 = ParityEngine::compute(&stats);
    assert_eq!(score1.value, dec!(100.0));
    assert_eq!(score1.level, ParityLevel::Perfect);

    stats.exact_match_count = 0;
    stats.critical_mismatch_count = 1000;
    let score2 = ParityEngine::compute(&stats);
    assert_eq!(score2.value, dec!(0.0));
    assert_eq!(score2.level, ParityLevel::Poor);
}

#[test]
fn test_health_score() {
    let mut stats = ShadowStatistics::new();
    stats.exact_match_count = 100;

    let parity = ParityEngine::compute(&stats);
    let validator = GoLiveValidator::new();

    let health = HealthEngine::evaluate(&parity, &validator);
    assert_eq!(health, ShadowHealth::Good); // capped because validator streaks = 0
}

#[test]
fn test_determinism_100k_iterations() {
    let mut validator = GoLiveValidator::new();
    let mut stats = ShadowStatistics::new();
    let thresholds = ParityThresholds::institutional();
    let engine = ComparisonEngine::new();

    for i in 0..100_000 {
        // Pseudo deterministic cycle logic without randomness
        let expected = dec!(100.0) + rust_decimal::Decimal::from(i % 10);
        let actual = expected; // exact match

        let comp = engine.compare(expected, actual, &thresholds);
        assert_eq!(comp, ComparisonState::ExactMatch);

        stats.exact_match_count = stats.exact_match_count.saturating_add(1);
        validator.process_parity_pass();
    }

    assert_eq!(stats.exact_match_count, 100_000);
    assert_eq!(validator.state, ValidatorState::Approved);
    assert_eq!(validator.consecutive_parity_streaks, 100_000);
}
