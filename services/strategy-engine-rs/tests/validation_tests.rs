use rust_decimal::Decimal;
use strategy_engine_rs::validation::determinism::{DeterminismValidator, DeterminismState};
use strategy_engine_rs::validation::replay::ReplayValidator;
use strategy_engine_rs::validation::events::ValidationEvent;
use strategy_engine_rs::validation::snapshot::ValidationSnapshot;
use strategy_engine_rs::validation::monte_carlo::MonteCarloValidator;
use strategy_engine_rs::validation::stress::StressValidator;
use strategy_engine_rs::validation::benchmark::BenchmarkEngine;
use strategy_engine_rs::validation::certification::{CertificationEngine, CertificationState};

#[test]
fn test_determinism_100k_iterations() {
    let validator = DeterminismValidator::new();
    let state = validator.execute(|mut v| {
        v += Decimal::ONE;
        v
    }, Decimal::ZERO);

    assert_eq!(state, DeterminismState::Deterministic);
}

#[test]
fn test_replay_correctness() {
    let events = vec![
        ValidationEvent::ValueAdded { amount: Decimal::new(10, 0) },
        ValidationEvent::Multiplied { factor: Decimal::new(2, 0) },
        ValidationEvent::ValueSubtracted { amount: Decimal::new(5, 0) },
    ];
    let snapshot = ValidationSnapshot {
        value: Decimal::new(15, 0),
    };

    let replay = ReplayValidator::new();
    assert!(replay.verify(&events, &snapshot));
}

#[test]
fn test_stress_scenarios() {
    let validator = StressValidator::new();
    assert!(validator.execute_extreme_scenarios());
}

#[test]
fn test_monte_carlo_bounds() {
    let validator = MonteCarloValidator::new();
    assert!(validator.verify_permutations());
}

#[test]
fn test_benchmark_bounds() {
    let mut benchmark = BenchmarkEngine::new();
    benchmark.track(&[Decimal::new(1, 0), Decimal::new(1, 0), Decimal::new(1, 0)]);
    assert!(benchmark.is_within_target());
    
    benchmark.track(&[Decimal::new(100, 0)]);
    assert!(!benchmark.is_within_target());
}

#[test]
fn test_certification_transitions() {
    let mut cert = CertificationEngine::new();
    assert_eq!(cert.state, CertificationState::NotCertified);
    
    cert.evaluate(true, true, true, true, true);
    assert_eq!(cert.state, CertificationState::Candidate);

    cert.evaluate(true, true, true, true, true);
    assert_eq!(cert.state, CertificationState::Certified);

    cert.evaluate(false, true, true, true, true);
    assert_eq!(cert.state, CertificationState::NotCertified);
}

#[test]
fn test_no_direct_rejected_to_certified() {
    let mut cert = CertificationEngine::new();
    cert.evaluate(true, true, true, true, true); // Candidate
    cert.evaluate(false, false, false, false, false); // Demoted to NotCertified
    cert.evaluate(true, true, true, true, true); // Should go to Candidate, not Certified
    assert_eq!(cert.state, CertificationState::Candidate);
}

#[test]
fn test_event_rebuild() {
    let event = ValidationEvent::ValueAdded { amount: Decimal::ONE };
    let cloned = event.clone();
    assert_eq!(event, cloned);
}

#[test]
fn test_snapshot_rebuild() {
    let snapshot = ValidationSnapshot { value: Decimal::ONE };
    let cloned = snapshot.clone();
    assert_eq!(snapshot, cloned);
}
