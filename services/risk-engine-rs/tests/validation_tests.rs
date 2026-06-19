use rust_decimal::Decimal;
use risk_engine::validation::certification::{CertificationState, RiskCertificationEngine};
use risk_engine::validation::determinism::DeterminismValidator;
use risk_engine::validation::replay::ReplayValidator;
use risk_engine::validation::stress::StressValidator;
use risk_engine::validation::benchmark::BenchmarkEngine;

#[tokio::test]
async fn test_determinism_100k_iterations() {
    let validator = DeterminismValidator::new();
    let result = validator.validate().expect("Determinism validation must not panic");
    
    assert!(result.identical_output);
    assert_eq!(result.iterations, 100_000);
}

#[tokio::test]
async fn test_replay_correctness() {
    let validator = ReplayValidator::new();
    let result = validator.validate().expect("Replay validation must not panic");

    assert!(result.exact_match);
}

#[tokio::test]
async fn test_benchmark_thresholds() {
    let validator = BenchmarkEngine::new();
    let result = validator.validate().expect("Benchmark validation must not panic");

    assert!(result.targets_met);
    // Average latency < 2ms constraint test
    assert!(result.average_latency_ms < Decimal::new(2, 0));
    // p99 latency < 10ms constraint test
    assert!(result.p99_latency_ms < Decimal::new(10, 0));
}

#[tokio::test]
async fn test_certification_transitions() {
    let engine = RiskCertificationEngine::new();

    // NotCertified -> Candidate
    let res = engine.certify(
        CertificationState::NotCertified,
        Decimal::new(100, 0), // agreement
        0,                    // panics
        false,                // corruption
        true,                 // determinism
        true,                 // replay
        true,                 // benchmark
    ).expect("Must not panic");

    assert_eq!(res.state, CertificationState::Candidate);
    assert!(res.reasons.is_empty());

    // Candidate -> Certified
    let res2 = engine.certify(
        CertificationState::Candidate,
        Decimal::new(100, 0), 0, false, true, true, true,
    ).expect("Must not panic");

    assert_eq!(res2.state, CertificationState::Certified);

    // Rejected -> Certified (Forbidden)
    let res3 = engine.certify(
        CertificationState::Rejected,
        Decimal::new(100, 0), 0, false, true, true, true,
    ).expect("Must not panic");

    assert_eq!(res3.state, CertificationState::Rejected);
    assert_eq!(res3.reasons[0], "Cannot transition from Rejected to Certified directly");

    // Must fail certification if agreement is <= 99%
    let res_fail = engine.certify(
        CertificationState::NotCertified,
        Decimal::new(99, 0), 0, false, true, true, true,
    ).expect("Must not panic");

    assert_eq!(res_fail.state, CertificationState::Rejected);
    assert!(res_fail.reasons.contains(&"Agreement 99 is not > 99%".to_string()));
}

#[tokio::test]
async fn test_stress_scenarios() {
    let validator = StressValidator::new();
    let result = validator.validate().expect("Stress validation must not panic");

    assert_eq!(result.panics, 0);
    assert!(!result.corruption_detected);
}
