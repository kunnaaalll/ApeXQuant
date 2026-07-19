use super::{
    benchmark::BenchmarkResult, certification::CertificationResult, determinism::DeterminismResult,
    monte_carlo::MonteCarloResult, parity::RiskParityResult, replay::ReplayResult,
    stress::StressResult,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    pub parity: RiskParityResult,
    pub determinism: DeterminismResult,
    pub replay: ReplayResult,
    pub monte_carlo: MonteCarloResult,
    pub stress: StressResult,
    pub benchmark: BenchmarkResult,
    pub certification: CertificationResult,
}

pub struct Reporter;

impl Default for Reporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter {
    pub fn new() -> Self {
        Self
    }

    /// Generates the markdown version of the validation report
    pub fn generate_markdown(
        &self,
        report: &ValidationReport,
    ) -> Result<String, crate::error::RiskError> {
        let md = format!(
            "# Risk Engine Validation Report\n\n\
            ## Certification Status\n\
            State: {:?}\n\
            \n\
            ## Parity\n\
            Agreement: {}%\n\
            \n\
            ## Determinism\n\
            Identical Output: {}\n\
            Iterations: {}\n\
            \n\
            ## Replay\n\
            Exact Match: {}\n\
            Events: {}\n\
            \n\
            ## Benchmark\n\
            Average Latency (ms): {}\n\
            P99 Latency (ms): {}\n\
            Allocations/sec: {}\n\
            Throughput (events/sec): {}\n\
            Targets Met: {}\n",
            report.certification.state,
            report.parity.agreement_percentage,
            report.determinism.identical_output,
            report.determinism.iterations,
            report.replay.exact_match,
            report.replay.event_count,
            report.benchmark.average_latency_ms,
            report.benchmark.p99_latency_ms,
            report.benchmark.allocations_per_sec,
            report.benchmark.throughput_events_per_sec,
            report.benchmark.targets_met,
        );
        Ok(md)
    }

    /// Generates the JSON version of the validation report
    pub fn generate_json(
        &self,
        report: &ValidationReport,
    ) -> Result<String, crate::error::RiskError> {
        // Since we forbid panic/unwrap, we map serialization errors
        serde_json::to_string_pretty(report)
            .map_err(|e| crate::error::RiskError::InternalError(e.to_string()))
    }
}
