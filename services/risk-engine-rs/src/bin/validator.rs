use risk_engine::{RiskEngine, config::RiskEngineConfig};
use risk_engine::validation::{parity, monte_carlo, stress, determinism, benchmark, report};
use tracing::info;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting Institutional-Grade Risk Engine Validation...");

    let config = RiskEngineConfig {
        shadow_mode: true,
        ..Default::default()
    };
    let engine = RiskEngine::new(config, None);

    info!("Running Parity tests against TS Risk Engine dataset...");
    let parity_res = parity::run_parity_validation(&engine).await;

    info!("Running Monte Carlo simulation (10,000 iterations)...");
    let mc_res = monte_carlo::run_monte_carlo_validation(&engine).await;

    info!("Injecting stress edge cases (consecutive losses, extreme ATR)...");
    let stress_res = stress::run_stress_validation(&engine).await;

    info!("Running Determinism validation (100,000 iterations)...");
    let det_res = determinism::run_determinism_validation(&engine).await;

    info!("Running Benchmark validations...");
    let bench_res = benchmark::run_benchmark_validation(&engine).await;

    info!("Aggregating results and generating certification reports...");
    report::generate_reports(&parity_res, &mc_res, &stress_res, &det_res, &bench_res);

    info!("Validation complete. Reports written to workspace root.");
}
