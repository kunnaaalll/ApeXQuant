use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationEvent {
    ParityValidated {
        score: Decimal,
    },
    DeterminismValidated {
        passed: bool,
    },
    ReplayValidated {
        passed: bool,
    },
    StressValidated {
        passed: bool,
    },
    BenchmarkUpdated {
        average_latency_ms: Decimal,
        p99_latency_ms: Decimal,
    },
    CertificationPromoted,
    CertificationDemoted,
}
