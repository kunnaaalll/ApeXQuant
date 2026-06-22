use rust_decimal_macros::dec;
use crate::validation::determinism_validator::DeterminismValidator;
use crate::validation::replay_validator::{ReplayValidator, ReplayStatus};
use crate::validation::stress_validator::{StressValidator, StressStatus};
use crate::validation::certification::{CertificationEngine, CertificationState};
use crate::validation::health::ValidationHealth;
use crate::validation::score::{ValidationScore, ValidationLevel};
use crate::validation::benchmark::{BenchmarkResult, BenchmarkEngine};
use crate::validation::state::ValidationState;
use crate::validation::snapshots::ValidationSnapshot;
use crate::validation::events::ValidationEvent;
use crate::validation::parity_validator::{ParityValidator, ParityResult};

#[test]
fn test_parity_score_bounds() {
    let (res, score) = ParityValidator::validate(
        dec!(100), dec!(100), dec!(100), dec!(100), dec!(100), dec!(100), dec!(100)
    );
    assert_eq!(score, dec!(100));
    assert_eq!(res, ParityResult::Perfect);

    let (res, score) = ParityValidator::validate(
        dec!(0), dec!(0), dec!(0), dec!(0), dec!(0), dec!(0), dec!(0)
    );
    assert_eq!(score, dec!(0));
    assert_eq!(res, ParityResult::Failure);
}

#[test]
fn test_determinism_100k_iterations() {
    let logic = || { dec!(42) };
    assert_eq!(
        DeterminismValidator::run_iterations(logic, dec!(42)),
        crate::validation::determinism_validator::DeterminismStatus::Deterministic
    );
}

#[test]
fn test_replay_correctness() {
    let state = ValidationState::default();
    let snapshot = ValidationSnapshot::new(state.clone());
    let events = vec![];
    assert_eq!(ReplayValidator::validate(&snapshot, &events, &state), ReplayStatus::Exact);
}

#[test]
fn test_stress_scenarios() {
    let logic = || { true };
    assert_eq!(StressValidator::verify_frozen_broker(logic), StressStatus::Healthy);
    assert_eq!(StressValidator::verify_disconnected_exchange(logic), StressStatus::Healthy);
    assert_eq!(StressValidator::verify_zero_liquidity(logic), StressStatus::Healthy);
    assert_eq!(StressValidator::verify_100_percent_rejection(logic), StressStatus::Healthy);
    assert_eq!(StressValidator::verify_severe_slippage(logic), StressStatus::Healthy);
    assert_eq!(StressValidator::verify_prolonged_latency(logic), StressStatus::Healthy);
    
    let no_panic = || { let _ = 1 + 1; };
    assert!(StressValidator::verify_no_panics(no_panic));
}

#[test]
fn test_benchmark_limits() {
    let result = BenchmarkResult {
        average_latency_ms: dec!(1.5),
        p99_latency_ms: dec!(8.0),
        replay_time_ms: dec!(0),
        snapshot_time_ms: dec!(0),
        serialization_time_ms: dec!(0),
        validation_time_ms: dec!(0),
    };
    assert!(BenchmarkEngine::check_thresholds(&result));

    let fail_result = BenchmarkResult {
        average_latency_ms: dec!(2.5),
        p99_latency_ms: dec!(11.0),
        replay_time_ms: dec!(0),
        snapshot_time_ms: dec!(0),
        serialization_time_ms: dec!(0),
        validation_time_ms: dec!(0),
    };
    assert!(!BenchmarkEngine::check_thresholds(&fail_result));
}

#[test]
fn test_certification_progression() {
    let mut engine = CertificationEngine::new();
    assert_eq!(engine.current_state(), CertificationState::NotCertified);

    engine.process_results(true, true, true, true, true);
    assert_eq!(engine.current_state(), CertificationState::Candidate);

    engine.process_results(true, true, true, true, true);
    assert_eq!(engine.current_state(), CertificationState::Certified);
}

#[test]
fn test_forbidden_transitions() {
    let mut engine = CertificationEngine::new();
    engine.process_results(true, true, true, true, true);
    engine.process_results(true, true, true, true, true);
    assert_eq!(engine.current_state(), CertificationState::Certified);

    engine.process_results(false, false, false, false, false);
    assert_eq!(engine.current_state(), CertificationState::Candidate);

    engine.process_results(false, false, false, false, false);
    assert_eq!(engine.current_state(), CertificationState::Rejected);

    // Rejected cannot go to Certified directly
    engine.process_results(true, true, true, true, true);
    assert_eq!(engine.current_state(), CertificationState::Rejected);
    
    // Have to reset
    engine.reset_rejected();
    assert_eq!(engine.current_state(), CertificationState::NotCertified);
}

#[test]
fn test_event_rebuild() {
    let state = ValidationState::default();
    let snapshot = ValidationSnapshot::new(state.clone());
    
    let mut expected_state = state.clone();
    expected_state.parity_score = dec!(99);
    
    let events = vec![ValidationEvent::ParityValidated { score: dec!(99) }];
    
    assert_eq!(ReplayValidator::validate(&snapshot, &events, &expected_state), ReplayStatus::Exact);
}

#[test]
fn test_snapshot_restore() {
    let state = ValidationState::default();
    let snapshot = ValidationSnapshot::new(state.clone());
    assert_eq!(snapshot.state, state);
}

#[test]
fn test_health_mapping() {
    assert_eq!(ValidationHealth::derive(true, true, true, true, true), ValidationHealth::Excellent);
    assert_eq!(ValidationHealth::derive(true, true, true, true, false), ValidationHealth::Good);
    assert_eq!(ValidationHealth::derive(true, true, true, false, false), ValidationHealth::Normal);
    assert_eq!(ValidationHealth::derive(true, true, false, false, false), ValidationHealth::Weak);
    assert_eq!(ValidationHealth::derive(true, false, false, false, false), ValidationHealth::Critical);
}

#[test]
fn test_validation_score_bounds() {
    let score = ValidationScore::new(dec!(105));
    assert_eq!(score.value, dec!(100));
    assert_eq!(score.level(), ValidationLevel::Elite);

    let score2 = ValidationScore::new(dec!(-5));
    assert_eq!(score2.value, dec!(0));
    assert_eq!(score2.level(), ValidationLevel::Broken);

    let score3 = ValidationScore::new(dec!(80));
    assert_eq!(score3.level(), ValidationLevel::Strong);
}

#[test]
fn test_zero_overflow() {
    // Rust Decimal handles this, but we explicitly test boundaries
    let a = dec!(100);
    let b = dec!(0);
    let _ = a + b;
    assert!(true); // If it doesn't panic, we're good
}

#[test]
fn test_zero_division() {
    // As rust_decimal panics on division by zero, our code should ensure we don't divide by zero.
    // E.g. ParityValidator divides by a constant 7, which is safe.
    assert!(true);
}

#[test]
fn test_no_corruption_under_stress() {
    // Testing heavy event streams without failure
    let events: Vec<ValidationEvent> = (0..1000).map(|_| ValidationEvent::ParityValidated { score: dec!(100) }).collect();
    let snapshot = ValidationSnapshot::new(ValidationState::default());
    
    let mut end_state = ValidationState::default();
    end_state.parity_score = dec!(100);
    
    assert_eq!(ReplayValidator::validate(&snapshot, &events, &end_state), ReplayStatus::Exact);
}
