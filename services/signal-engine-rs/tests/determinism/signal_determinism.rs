//! Determinism Tests for Signal Engine
//!
//! Validates that the Signal Engine produces identical outputs for identical inputs
//! across thousands of executions. Zero variance is expected.

use signal_engine::parity::SignalEngine;
use signal_engine::replay::{GoldenDataset, TestScenario};
use std::collections::HashSet;

const ITERATIONS: u32 = 1000;
const DETERMINISM_THRESHOLD: f64 = 100.0; // Must be 100%

/// Run determinism test on a single scenario
async fn test_scenario_determinism(scenario: &TestScenario) -> DeterminismTestResult {
    let mut engine = SignalEngine::new(Default::default()).await.unwrap();

    let mut results = Vec::with_capacity(ITERATIONS as usize);
    let mut hashes = HashSet::new();

    for i in 0..ITERATIONS {
        let result = engine.analyze(scenario.candles.clone()).await.unwrap();

        // Hash the result
        let hash = hash_result(&result);
        hashes.insert(hash);

        results.push(result);

        // Progress indicator
        if i % 100 == 0 {
            println!("  Iteration {}/{}", i, ITERATIONS);
        }
    }

    let unique_count = hashes.len();
    let determinism_rate = if unique_count == 1 {
        100.0
    } else {
        ((ITERATIONS as usize - unique_count + 1) as f64 / ITERATIONS as f64) * 100.0
    };

    // Compare key fields across all results
    let first = &results[0];
    let all_match = results.iter().all(|r| {
        r.direction == first.direction
            && r.confidence == first.confidence
            && r.confluence_score == first.confluence_score
            && r.entry_zone == first.entry_zone
            && r.stop_price == first.stop_price
            && r.target_price == first.target_price
    });

    DeterminismTestResult {
        scenario_id: scenario.scenario_id.clone(),
        total_iterations: ITERATIONS,
        unique_hashes: unique_count as u32,
        determinism_rate,
        all_fields_match: all_match,
        first_result: results[0].clone(),
        last_result: results[results.len() - 1].clone(),
    }
}

fn hash_result(result: &signal_engine::signals::SignalResult) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Hash only the deterministic fields
    result.direction.hash(&mut hasher);
    result.confidence.to_bits().hash(&mut hasher);
    result.confluence_score.to_bits().hash(&mut hasher);
    if let Some(ref sl) = result.stop_price {
        sl.to_bits().hash(&mut hasher);
    }
    result.target_price.to_bits().hash(&mut hasher);

    format!("{:x}", hasher.finish())
}

#[derive(Debug)]
struct DeterminismTestResult {
    scenario_id: String,
    total_iterations: u32,
    unique_hashes: u32,
    determinism_rate: f64,
    all_fields_match: bool,
    first_result: signal_engine::signals::SignalResult,
    last_result: signal_engine::signals::SignalResult,
}

/// Main determinism test runner
#[tokio::test]
async fn test_signal_engine_determinism() {
    println!("\n");
    println!("==============================================");
    println!("SIGNAL ENGINE DETERMINISM VALIDATION");
    println!("==============================================");
    println!("Iterations per scenario: {}", ITERATIONS);
    println!("Determinism threshold: {}%", DETERMINISM_THRESHOLD);
    println!("\n");

    // Create or load dataset
    let dataset = create_test_dataset();

    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    for scenario in &dataset.scenarios {
        println!("Testing scenario: {} - {}", scenario.scenario_id, scenario.name);

        let result = test_scenario_determinism(scenario).await;

        println!("  Unique hashes: {} ", result.unique_hashes);
        println!("  Determinism rate: {:.2}%", result.determinism_rate);
        println!("  All fields match: {}", result.all_fields_match);

        if result.determinism_rate >= DETERMINISM_THRESHOLD && result.all_fields_match {
            println!("  ✅ PASS");
            passed += 1;
        } else {
            println!("  ❌ FAIL");
            failed += 1;
            failures.push(result);
        }
        println!();
    }

    // Summary
    println!("\n");
    println!("==============================================");
    println!("SUMMARY");
    println!("==============================================");
    println!("Total scenarios: {}", dataset.scenarios.len());
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!();

    if failed > 0 {
        println!("Failed scenarios:");
        for f in &failures {
            println!("  - {}: {:.2}% determinism", f.scenario_id, f.determinism_rate);
        }
    }

    // Zero tolerance - any failure is critical
    assert_eq!(
        failed, 0,
        "{} scenarios failed determinism validation. Signal Engine must be 100% deterministic before go-live.",
        failed
    );

    println!("✅ All scenarios passed determinism validation!");
    println!("System ready for go-live consideration.");
}

/// Create synthetic test dataset for determinism testing
fn create_test_dataset() -> GoldenDataset {
    use signal_engine::market_data::Candle;
    use signal_engine::replay::{ScenarioCategory, TestScenario};
    use chrono::Duration;

    let mut dataset = GoldenDataset::new("determinism_test".to_string());

    // Create test scenarios covering different market conditions
    let scenarios = vec![
        ("uptrend", ScenarioCategory::TrendingUp),
        ("downtrend", ScenarioCategory::TrendingDown),
        ("ranging", ScenarioCategory::Ranging),
        ("breakout", ScenarioCategory::Breakout),
        ("highvol", ScenarioCategory::HighVolatility),
    ];

    for (i, (name, category)) in scenarios.iter().enumerate() {
        let base = 1.0850;
        let mut candles = Vec::with_capacity(100);

        for j in 0..100 {
            let (open, close) = match category {
                ScenarioCategory::TrendingUp => (
                    base + j as f64 * 0.0001,
                    base + (j + 1) as f64 * 0.0001,
                ),
                ScenarioCategory::TrendingDown => (
                    base - j as f64 * 0.0001,
                    base - (j + 1) as f64 * 0.0001,
                ),
                ScenarioCategory::Ranging => {
                    let val = base + (j % 10) as f64 * 0.0001 - 0.0005;
                    (val, val + 0.0001)
                }
                _ => (
                    base + (rand::random::<f64>() - 0.5) * 0.001,
                    base + (rand::random::<f64>() - 0.5) * 0.001,
                ),
            };

            let candle = Candle {
                timestamp: Utc::now() + Duration::minutes(j as i64 * 15),
                open,
                high: open.max(close) + 0.0001,
                low: open.min(close) - 0.0001,
                close,
                volume: 1000 + j as u64 * 10,
            };
            candles.push(candle);
        }

        let scenario = TestScenario {
            scenario_id: format!("det_{:03}", i),
            name: format!("Determinism Test {}", name),
            description: format!("Determinism test for {} market", name),
            category: *category,
            symbol: "EURUSD".to_string(),
            timeframe: "M15".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now() + Duration::hours(25),
            candles,
            expected_signals: vec![],
            metadata: std::collections::HashMap::new(),
        };

        dataset.add_scenario(scenario);
    }

    dataset
}
