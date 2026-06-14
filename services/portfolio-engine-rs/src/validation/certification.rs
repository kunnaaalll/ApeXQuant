use super::parity::{PortfolioParityResult, ParityState};
use super::benchmark::BenchmarkReport;
use super::stress::StressReport;
use super::determinism::{DeterminismReport, DeterminismState};
use super::monte_carlo::MonteCarloReport;
use super::validator::ReplayResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertificationLevel {
    Fail,
    Warning,
    Pass,
    Certified,
}

#[derive(Debug, Clone)]
pub struct PortfolioCertification {
    pub level: CertificationLevel,
    pub parity_result: PortfolioParityResult,
    pub benchmark_report: BenchmarkReport,
    pub stress_report: StressReport,
    pub determinism_report: DeterminismReport,
    pub monte_carlo_report: MonteCarloReport,
    pub replay_result: ReplayResult,
}

pub struct PortfolioCertificationEngine;

impl PortfolioCertificationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn certify(
        &self,
        parity: PortfolioParityResult,
        benchmark: BenchmarkReport,
        stress: StressReport,
        determinism: DeterminismReport,
        monte_carlo: MonteCarloReport,
        replay: ReplayResult,
    ) -> PortfolioCertification {
        // Criteria:
        // State agreement >99%
        // Recommendation agreement >95%
        // Analytics agreement >95%
        // Health drift <5% (agreement > 95%)
        // Quality drift <5% (agreement > 95%)
        // Drawdown drift <2% (agreement > 98%)
        // Heat drift <2% (agreement > 98%)
        // Replay divergence = 0
        // Determinism failures = 0
        // Panics = 0
        // Memory leaks = 0
        // P99 latency <20 ms

        let meets_parity = parity.state_agreement_pct > 99.0 &&
                           parity.recommendation_agreement_pct > 95.0 &&
                           parity.analytics_agreement_pct > 95.0 &&
                           parity.health_agreement_pct > 95.0 &&
                           parity.quality_agreement_pct > 95.0 &&
                           parity.drawdown_agreement_pct > 98.0 &&
                           parity.heat_agreement_pct > 98.0;

        let meets_benchmark = benchmark.p99_latency.as_millis() < 20 &&
                              benchmark.memory_leaks_detected == 0;

        let meets_stress = stress.panics_detected == 0 &&
                           stress.data_corruption_detected == 0 &&
                           stress.race_conditions_detected == 0;

        let meets_determinism = determinism.overall_state == DeterminismState::Pass &&
                                determinism.divergence_count == 0;

        let meets_replay = !replay.drift_detected && replay.exact_match;

        let meets_monte_carlo = monte_carlo.survival_rate_pct > 99.0;

        let level = if meets_parity && meets_benchmark && meets_stress && 
                       meets_determinism && meets_replay && meets_monte_carlo {
            CertificationLevel::Certified
        } else if meets_parity && meets_stress && meets_replay {
            CertificationLevel::Pass
        } else if meets_stress {
            CertificationLevel::Warning
        } else {
            CertificationLevel::Fail
        };

        PortfolioCertification {
            level,
            parity_result: parity,
            benchmark_report: benchmark,
            stress_report: stress,
            determinism_report: determinism,
            monte_carlo_report: monte_carlo,
            replay_result: replay,
        }
    }
}
