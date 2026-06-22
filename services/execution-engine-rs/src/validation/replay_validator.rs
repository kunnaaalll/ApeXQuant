use crate::validation::snapshots::ValidationSnapshot;
use crate::validation::events::ValidationEvent;
use crate::validation::state::ValidationState;
use crate::validation::health::ValidationHealth;
use crate::validation::certification::CertificationState;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayStatus {
    Exact,
    Mismatch,
    Corrupted,
}

pub struct ReplayValidator;

impl ReplayValidator {
    pub fn validate(snapshot: &ValidationSnapshot, events: &[ValidationEvent], final_state: &ValidationState) -> ReplayStatus {
        let mut current_state = snapshot.state.clone();

        for event in events {
            match event {
                ValidationEvent::ParityValidated { score } => {
                    current_state.parity_score = *score;
                }
                ValidationEvent::DeterminismValidated { passed } => {
                    if !*passed { current_state.health = ValidationHealth::Critical; }
                }
                ValidationEvent::ReplayValidated { passed } => {
                    if !*passed { current_state.health = ValidationHealth::Critical; }
                }
                ValidationEvent::StressValidated { passed } => {
                    if !*passed { current_state.health = ValidationHealth::Critical; }
                }
                ValidationEvent::BenchmarkUpdated { average_latency_ms, p99_latency_ms } => {
                    current_state.benchmark_metrics = Some(crate::validation::benchmark::BenchmarkResult {
                        average_latency_ms: *average_latency_ms,
                        p99_latency_ms: *p99_latency_ms,
                        replay_time_ms: dec!(0),
                        snapshot_time_ms: dec!(0),
                        serialization_time_ms: dec!(0),
                        validation_time_ms: dec!(0),
                    });
                }
                ValidationEvent::CertificationPromoted => {
                    current_state.certification_state = match current_state.certification_state {
                        CertificationState::NotCertified => CertificationState::Candidate,
                        CertificationState::Candidate => CertificationState::Certified,
                        CertificationState::Certified => CertificationState::Certified,
                        CertificationState::Rejected => CertificationState::Rejected,
                    };
                }
                ValidationEvent::CertificationDemoted => {
                    current_state.certification_state = CertificationState::Rejected;
                }
            }
        }

        if &current_state == final_state {
            ReplayStatus::Exact
        } else {
            ReplayStatus::Mismatch
        }
    }
}
