use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::validation::certification::CertificationState;
use crate::validation::health::ValidationHealth;
use crate::validation::score::ValidationScore;
use crate::validation::benchmark::BenchmarkResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationState {
    pub score: ValidationScore,
    pub health: ValidationHealth,
    pub certification_state: CertificationState,
    pub benchmark_metrics: Option<BenchmarkResult>,
    pub parity_score: Decimal,
}

impl Default for ValidationState {
    fn default() -> Self {
        Self {
            score: ValidationScore::default(),
            health: ValidationHealth::Critical,
            certification_state: CertificationState::NotCertified,
            benchmark_metrics: None,
            parity_score: dec!(0),
        }
    }
}
