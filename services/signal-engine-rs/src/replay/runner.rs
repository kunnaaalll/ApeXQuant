//! Replay Runner - Execute replay scenarios and collect results

use super::*;
use crate::parity::{ComparisonEngine, SignalComparisonRecord, SignalOutput};
use crate::signals::{SignalEngine, SignalResult};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Configuration for replay runs
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    pub speed: ReplaySpeed,
    pub verification: VerificationLevel,
    pub collect_metrics: bool,
    pub parallel_scenarios: usize,
    pub stop_on_failure: bool,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            speed: ReplaySpeed::Maximum,
            verification: VerificationLevel::Full,
            collect_metrics: true,
            parallel_scenarios: 1,
            stop_on_failure: true,
        }
    }
}

/// Result of a replay run
#[derive(Debug, Clone)]
pub struct ReplayResult {
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub total_scenarios: usize,
    pub completed_scenarios: usize,
    pub failed_scenarios: Vec<FailedScenario>,
    pub comparisons: Vec<SignalComparisonRecord>,
    pub metrics: ReplayMetrics,
}

/// Failed scenario information
#[derive(Debug, Clone)]
pub struct FailedScenario {
    pub scenario_id: String,
    pub failure_reason: String,
    pub timestamp: DateTime<Utc>,
}

/// Replay engine for executing scenarios
pub struct ReplayEngine {
    signal_engine: Option<Arc<SignalEngine>>,
    ts_reference: Option<TSReference>,
    config: ReplayConfig,
}

/// TypeScript reference interface
pub struct TSReference {
    // In practice this would be a gRPC client or HTTP client
    endpoint: String,
}

impl ReplayEngine {
    /// Create a new replay engine
    pub fn new(config: ReplayConfig) -> Self {
        Self {
            signal_engine: None,
            ts_reference: None,
            config,
        }
    }

    /// Attach a Rust signal engine
    pub fn with_signal_engine(mut self, engine: Arc<SignalEngine>) -> Self {
        self.signal_engine = Some(engine);
        self
    }

    /// Attach TypeScript reference endpoint
    pub fn with_ts_reference(mut self, endpoint: String) -> Self {
        self.ts_reference = Some(TSReference { endpoint });
        self
    }

    /// Run a single scenario and get determinism results
    pub async fn run_determinism_test(
        &self,
        scenario: &TestScenario,
        iterations: u32,
    ) -> Result<DeterminismResult> {
        let engine = self
            .signal_engine
            .as_ref()
            .ok_or_else(|| ReplayError::ReplayFailed("No signal engine attached".to_string()))?;

        let mut results: Vec<SignalResult> = Vec::with_capacity(iterations as usize);
        let mut latencies: Vec<Duration> = Vec::with_capacity(iterations as usize);

        for _ in 0..iterations {
            let start = Instant::now();
            let result = self.run_scenario(engine, scenario).await?;
            let latency = start.elapsed();

            results.push(result);
            latencies.push(latency);
        }

        // Check determinism
        let is_deterministic = self.check_determinism(&results);
        let hash = self.hash_results(&results);

        // Calculate latency statistics
        let latencies_ms: Vec<f64> = latencies.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
        let avg_latency = latencies_ms.iter().sum::<f64>() / latencies_ms.len() as f64;
        let max_latency = *latencies_ms.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(&0.0);
        let min_latency = *latencies_ms.iter().min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(&0.0);

        Ok(DeterminismResult {
            scenario_id: scenario.scenario_id.clone(),
            iterations,
            is_deterministic,
            hash,
            first_result: results.first().cloned(),
            last_result: results.last().cloned(),
            avg_latency_ms: avg_latency,
            min_latency_ms: min_latency,
            max_latency_ms: max_latency,
            results_sample: if iterations > 10 {
                results.into_iter().take(10).collect()
            } else {
                results
            },
        })
    }

    /// Run full parity comparison between Rust and TypeScript
    pub async fn run_parity_test(
        &self,
        scenario: &TestScenario,
    ) -> Result<ParityReplayResult> {
        let engine = self
            .signal_engine
            .as_ref()
            .ok_or_else(|| ReplayError::ReplayFailed("No signal engine attached".to_string()))?;

        let ts = self
            .ts_reference
            .as_ref()
            .ok_or_else(|| ReplayError::ReplayFailed("No TS reference attached".to_string()))?;

        // Get Rust signal
        let rust_start = Instant::now();
        let rust_signal = self.run_scenario(engine, scenario).await?;
        let rust_latency = rust_start.elapsed();

        // Get TypeScript signal (would call actual TS endpoint in production)
        let ts_start = Instant::now();
        let ts_signal = self.query_ts_reference(ts, scenario).await?;
        let ts_latency = ts_start.elapsed();

        // Convert signals to comparable outputs
        let rust_output = self.signal_to_output(&rust_signal, scenario);
        let ts_output = self.signal_to_output(&ts_signal, scenario);

        // Compare
        let comparison_engine = ComparisonEngine::new(&Default::default());
        let comparison = comparison_engine.compare(ts_output, rust_output)?;

        Ok(ParityReplayResult {
            scenario_id: scenario.scenario_id.clone(),
            comparison,
            rust_latency_ms: rust_latency.as_secs_f64() * 1000.0,
            ts_latency_ms: ts_latency.as_secs_f64() * 1000.0,
        })
    }

    /// Run throughput benchmark on a scenario
    pub async fn run_benchmark(
        &self,
        scenario: &TestScenario,
        config: BenchmarkConfig,
    ) -> Result<BenchmarkResult> {
        let engine = self
            .signal_engine
            .as_ref()
            .ok_or_else(|| ReplayError::ReplayFailed("No signal engine attached".to_string()))?;

        // Warmup
        for _ in 0..config.warmup_iterations {
            let _ = self.run_scenario(engine, scenario).await?;
        }

        // Measurement
        let start = Instant::now();
        for _ in 0..config.measurement_iterations {
            let _ = self.run_scenario(engine, scenario).await?;
        }
        let total_duration = start.elapsed();

        let avg_duration_ms =
            total_duration.as_secs_f64() * 1000.0 / config.measurement_iterations as f64;
        let throughput = config.measurement_iterations as f64 / total_duration.as_secs_f64();

        Ok(BenchmarkResult {
            iterations: config.measurement_iterations,
            total_duration_ms: total_duration.as_secs_f64() * 1000.0,
            avg_duration_ms,
            throughput_per_second: throughput,
        })
    }

    /// Run entire dataset through replay
    pub async fn run_dataset(&self, dataset: &GoldenDataset) -> Result<ReplayResult> {
        let started_at = Utc::now();
        let mut results = Vec::new();
        let mut failed = Vec::new();

        for scenario in &dataset.scenarios {
            match self.run_determinism_test(scenario, 1).await {
                Ok(result) => {
                    // Would add to results
                    results.push(result);
                }
                Err(e) => {
                    failed.push(FailedScenario {
                        scenario_id: scenario.scenario_id.clone(),
                        failure_reason: e.to_string(),
                        timestamp: Utc::now(),
                    });

                    if self.config.stop_on_failure {
                        break;
                    }
                }
            }
        }

        let completed_at = Utc::now();

        Ok(ReplayResult {
            started_at,
            completed_at,
            total_scenarios: dataset.scenarios.len(),
            completed_scenarios: results.len(),
            failed_scenarios: failed,
            comparisons: Vec::new(), // Would populate from parity tests
            metrics: ReplayMetrics::default(),
        })
    }

    /// Internal: run a single scenario through the signal engine
    async fn run_scenario(
        &self,
        _engine: &SignalEngine,
        scenario: &TestScenario,
    ) -> Result<SignalResult> {
        // In practice, would feed candles to engine step by step
        // For now, return a placeholder result

        let _last_candle = scenario.candles.last().ok_or_else(|| {
            ReplayError::InvalidFormat("Scenario has no candles".to_string())
        })?;

        // This is a placeholder - actual implementation would call signal engine
        Err(ReplayError::ReplayFailed(
            "Signal engine integration not implemented".to_string(),
        ))
    }

    /// Internal: query TypeScript reference
    async fn query_ts_reference(&self, _ts: &TSReference, scenario: &TestScenario) -> Result<SignalResult> {
        // In production, this would make a gRPC/HTTP call to the TypeScript engine
        // For now, return expected signal if available

        if let Some(expected) = scenario.expected_signals.first() {
            Ok(SignalResult {
                direction: match expected.direction.as_str() {
                    "LONG" => crate::signals::SignalDirection::Long,
                    "SHORT" => crate::signals::SignalDirection::Short,
                    _ => crate::signals::SignalDirection::NoSignal,
                },
                confidence: expected.confidence,
                // ... other fields
                ..Default::default()
            })
        } else {
            Err(ReplayError::ReplayFailed(
                "No expected signal in scenario".to_string(),
            ))
        }
    }

    /// Convert SignalResult to SignalOutput for comparison
    fn signal_to_output(&self, result: &SignalResult, scenario: &TestScenario) -> SignalOutput {
        SignalOutput {
            timestamp: Utc::now(),
            symbol: scenario.symbol.clone(),
            timeframe: scenario.timeframe.clone(),
            direction: crate::parity::SignalDirection::Neutral, // Map from SignalDirection
            confidence: result.confidence,
            confluence_score: 0.0,
            entry_price: result.entry_zone.map(|z| z.center()),
            stop_loss: result.stop_price,
            take_profit: Some(result.target_price),
            patterns: Vec::new(),
            regime: "UNKNOWN".to_string(),
            session: "UNKNOWN".to_string(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Check if all results are identical
    fn check_determinism(&self, results: &[SignalResult]) -> bool {
        if results.len() < 2 {
            return true;
        }

        let first = &results[0];
        results.iter().all(|r| {
            // Compare key fields
            r.direction == first.direction
                && r.confidence == first.confidence
                && r.stop_price == first.stop_price
                && r.target_price == first.target_price
        })
    }

    /// Generate a hash of results for comparison
    fn hash_results(&self, results: &[SignalResult]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        results.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Result of a determinism test
#[derive(Debug, Clone)]
pub struct DeterminismResult {
    pub scenario_id: String,
    pub iterations: u32,
    pub is_deterministic: bool,
    pub hash: String,
    pub first_result: Option<SignalResult>,
    pub last_result: Option<SignalResult>,
    pub avg_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub results_sample: Vec<SignalResult>,
}

/// Result of a parity replay
#[derive(Debug, Clone)]
pub struct ParityReplayResult {
    pub scenario_id: String,
    pub comparison: SignalComparisonRecord,
    pub rust_latency_ms: f64,
    pub ts_latency_ms: f64,
}

/// Result of a benchmark run
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub iterations: u32,
    pub total_duration_ms: f64,
    pub avg_duration_ms: f64,
    pub throughput_per_second: f64,
}

/// Stress test runner
pub struct StressTestRunner {
    config: StressTestConfig,
}

impl StressTestRunner {
    /// Create a new stress test runner
    pub fn new(config: StressTestConfig) -> Self {
        Self { config }
    }

    /// Run stress test
    pub async fn run(&self) -> Result<StressTestResult> {
        let start = Instant::now();
        let mut completed_candles = 0usize;
        let mut errors = Vec::new();

        // Run concurrent replays
        let (tx, mut rx) = mpsc::channel(self.config.concurrent_replays);

        // Spawn workers
        for i in 0..self.config.concurrent_replays {
            let tx = tx.clone();
            let candles_per_worker = self.config.total_candles / self.config.concurrent_replays;

            tokio::spawn(async move {
                for _ in 0..candles_per_worker {
                    // Process candle
                    // In practice, would call signal engine
                    if let Err(e) = Self::process_dummy_candle().await {
                        let _ = tx.send((i, e.to_string())).await;
                    }
                }
            });
        }

        // Collect results until timeout
        let timeout = Duration::from_secs(self.config.target_duration_seconds + 300);
        let deadline = Instant::now() + timeout;

        while Instant::now() < deadline && completed_candles < self.config.total_candles {
            match tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
                Ok(Some((_, error))) => errors.push(error),
                Ok(None) => break,
                Err(_) => continue,
            }
            completed_candles += 1;
        }

        let duration = start.elapsed();
        let throughput = completed_candles as f64 / duration.as_secs_f64();

        Ok(StressTestResult {
            duration_seconds: duration.as_secs_f64(),
            completed_candles,
            target_candles: self.config.total_candles,
            throughput_per_second: throughput,
            errors,
            memory_stable: true, // Would check system memory
            no_panics: true,
        })
    }

    async fn process_dummy_candle() -> Result<()> {
        // Placeholder - actual implementation would feed to signal engine
        tokio::task::yield_now().await;
        Ok(())
    }
}

/// Stress test result
#[derive(Debug, Clone)]
pub struct StressTestResult {
    pub duration_seconds: f64,
    pub completed_candles: usize,
    pub target_candles: usize,
    pub throughput_per_second: f64,
    pub errors: Vec<String>,
    pub memory_stable: bool,
    pub no_panics: bool,
}

impl StressTestResult {
    /// Check if stress test passed
    pub fn passed(&self) -> bool {
        self.memory_stable && self.no_panics && self.errors.is_empty()
    }
}
