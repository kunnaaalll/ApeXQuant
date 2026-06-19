use crate::validation::certification::CertificationEngine;
use crate::validation::benchmark::BenchmarkEngine;

#[derive(Debug, Clone)]
pub struct ValidationReporter;

impl ValidationReporter {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_report(
        &self,
        certification: &CertificationEngine,
        benchmark: &BenchmarkEngine,
    ) -> String {
        format!(
            "# Validation Report\n\n\
            ## Certification State\n\
            State: {:?}\n\n\
            ## Benchmarks\n\
            - Average Latency: {} ms\n\
            - P99 Latency: {} ms\n",
            certification.state,
            benchmark.average_latency,
            benchmark.p99_latency
        )
    }
}

impl Default for ValidationReporter {
    fn default() -> Self {
        Self::new()
    }
}
