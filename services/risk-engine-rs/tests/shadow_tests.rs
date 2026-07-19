#![allow(warnings, clippy::all, deprecated)]
use chrono::Utc;
use risk_engine::shadow::comparison::{ComparisonEngine, ComparisonState};
use risk_engine::shadow::drift::{DriftEngine, DriftSeverity};
use risk_engine::shadow::statistics::{StatisticsEngine, StatisticsWindow};
use risk_engine::shadow::validator::{GoLiveValidator, ValidatorState};
use risk_engine::shadow::{LegacyRiskState, RustRiskState, ShadowSnapshot};
use rust_decimal::Decimal;

fn mock_legacy_state() -> LegacyRiskState {
    LegacyRiskState {
        drawdown: Decimal::new(10, 2),
        exposure: Decimal::new(5000, 2),
        correlation: Decimal::new(80, 2),
        hidden_leverage: Decimal::new(120, 2),
        var: Decimal::new(1000, 2),
        expected_shortfall: Decimal::new(1500, 2),
        circuit_breakers_tripped: 0,
        recommendation_code: 1,
        stress_assessment: 0,
    }
}

fn mock_rust_state() -> RustRiskState {
    RustRiskState {
        drawdown: Decimal::new(10, 2),
        exposure: Decimal::new(5000, 2),
        correlation: Decimal::new(80, 2),
        hidden_leverage: Decimal::new(120, 2),
        var: Decimal::new(1000, 2),
        expected_shortfall: Decimal::new(1500, 2),
        circuit_breakers_tripped: 0,
        recommendation_code: 1,
        stress_assessment: 0,
    }
}

#[test]
fn test_comparison_symmetry() {
    let legacy = mock_legacy_state();
    let rust = mock_rust_state();
    assert_eq!(
        ComparisonEngine::compare(&legacy, &rust),
        ComparisonState::ExactMatch
    );
}

#[test]
fn test_drift_bounds() {
    let zero = Decimal::ZERO;
    let small = Decimal::new(1, 2);
    let large = Decimal::new(1000, 0);

    let rel_zero = DriftEngine::measure_relative(zero, zero);
    assert_eq!(rel_zero, Decimal::ZERO);

    let rel_inf = DriftEngine::measure_relative(zero, small);
    assert_eq!(rel_inf, Decimal::ONE); // Bounded to 1.0 (100%)

    let rel_large = DriftEngine::measure_relative(small, large);
    assert_eq!(rel_large, Decimal::ONE); // Also bounded to 1.0
}

#[test]
fn test_statistics_agreement_bounds() {
    let mut stats = StatisticsWindow::new();
    assert_eq!(stats.agreement_percentage(), Decimal::new(100, 0));

    stats.record(&ComparisonState::Mismatch);
    assert_eq!(stats.agreement_percentage(), Decimal::ZERO);

    stats.record(&ComparisonState::ExactMatch);
    assert_eq!(stats.agreement_percentage(), Decimal::new(50, 0));
}

#[test]
fn test_validator_transitions() {
    let mut validator = GoLiveValidator::new();
    assert_eq!(validator.state, ValidatorState::NotReady);

    validator.process(&ComparisonState::Critical);
    assert_eq!(validator.state, ValidatorState::Rejected);

    for _ in 0..99 {
        validator.process(&ComparisonState::ExactMatch);
        assert_eq!(validator.state, ValidatorState::Rejected);
    }

    validator.process(&ComparisonState::ExactMatch);
    assert_eq!(validator.state, ValidatorState::Monitoring);

    for _ in 0..899 {
        validator.process(&ComparisonState::ExactMatch);
    }
    assert_eq!(validator.state, ValidatorState::Monitoring);
    validator.process(&ComparisonState::ExactMatch);
    assert_eq!(validator.state, ValidatorState::Candidate);

    for _ in 0..8999 {
        validator.process(&ComparisonState::ExactMatch);
    }
    assert_eq!(validator.state, ValidatorState::Candidate);
    validator.process(&ComparisonState::ExactMatch);
    assert_eq!(validator.state, ValidatorState::Approved);
}

#[test]
fn test_event_replay_snapshot_reconstruction() {
    let snapshot = ShadowSnapshot {
        timestamp: Utc::now(),
        legacy_state: mock_legacy_state(),
        rust_state: mock_rust_state(),
    };

    let serialized = serde_json::to_string(&snapshot).expect("Serialize ok");
    let deserialized: ShadowSnapshot = serde_json::from_str(&serialized).expect("Deserialize ok");

    assert_eq!(snapshot.legacy_state, deserialized.legacy_state);
    assert_eq!(snapshot.rust_state, deserialized.rust_state);
}

#[test]
fn test_determinism() {
    let legacy = mock_legacy_state();
    let rust = mock_rust_state();
    let mut stats = StatisticsEngine::new();
    let mut validator = GoLiveValidator::new();

    for _ in 0..100_000 {
        let cmp = ComparisonEngine::compare(&legacy, &rust);
        assert_eq!(cmp, ComparisonState::ExactMatch);

        let abs_drift = DriftEngine::measure_absolute(legacy.drawdown, rust.drawdown);
        let rel_drift = DriftEngine::measure_relative(legacy.drawdown, rust.drawdown);

        assert_eq!(abs_drift, Decimal::ZERO);
        assert_eq!(rel_drift, Decimal::ZERO);

        let severity = DriftEngine::classify(abs_drift, rel_drift);
        assert_eq!(severity, DriftSeverity::Normal);

        stats.record(&cmp);
        validator.process(&cmp);
    }

    assert_eq!(validator.state, ValidatorState::Approved);
    assert_eq!(stats.daily.total_comparisons, 100_000);
    assert_eq!(stats.daily.agreement_percentage(), Decimal::new(100, 0));
}
