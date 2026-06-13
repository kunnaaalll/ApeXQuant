# Signal Engine V1 - Test Plan

## Philosophy

The Signal Engine must be **deterministic** and **explainable**. Every test enforces these properties.

## Test Categories

### 1. Unit Tests

**Goal:** Verify individual modules in isolation.

#### Structure Module Tests

```rust
#[test]
fn swing_detection_three_bar_pivot() {
    // Given: Series with clear swing high
    let candles = vec![
        candle!(low: 1.0, high: 1.5, close: 1.2),  // -3
        candle!(low: 1.1, high: 1.6, close: 1.3),  // -2
        candle!(low: 1.2, high: 2.0, close: 1.8),  // -1 (pivot)
        candle!(low: 1.5, high: 1.8, close: 1.6),  // 0
        candle!(low: 1.4, high: 1.7, close: 1.5),  // +1
    ];

    // When: Detect swings with pivot=3
    let swings = SwingDetector::detect(&candles, 3);

    // Then: Swing high at index -1
    assert_eq!(swings.highs.len(), 1);
    assert_eq!(swings.highs[0].index, 2);
    assert_eq!(swings.highs[0].price, dec!(2.0));
}

#[test]
fn trend_uptrend_higher_highs() {
    // Given: HH, HL sequence
    let structure = vec![
        swing_low!(1.0),   // LL
        swing_high!(1.5),  // LH
        swing_low!(1.2),   // HL - trend change
        swing_high!(1.8),  // HH - uptrend confirmed
        swing_low!(1.4),   // HL
        swing_high!(2.0),  // HH
    ];

    // When: Classify trend
    let trend = TrendDetector::classify(&structure);

    // Then: Uptrend
    assert_eq!(trend.direction, TrendDirection::Up);
}
```

#### SMC Module Tests

```rust
#[test]
fn bos_bullish_breaks_lower_high() {
    // Given: Established downtrend with lower high
    let context = MarketContext::downtrend()
        .with_swing_high(2, dec!(1.5000)); // Lower high

    let candles = vec![
        candle!(high: 1.4900, close: 1.4850), // Approach
        candle!(high: 1.5050, close: 1.5020), // Break above
    ];

    // When: Detect BOS
    let bos = BosDetector::detect(&candles, &context);

    // Then: Bullish BOS detected
    assert!(bos.is_some());
    assert_eq!(bos.unwrap().direction, BosDirection::Bullish);
}

#[test]
fn order_block_bullish_last_bearish_before_impulse() {
    // Given: Bearish candle followed by bullish impulse
    let candles = vec![
        candle!(open: 1.5000, high: 1.5100, low: 1.4900, close: 1.4950), // Bearish
        candle!(open: 1.4950, high: 1.5200, low: 1.4900, close: 1.5180), // Bullish impulse
        candle!(open: 1.5180, high: 1.5300, low: 1.5150, close: 1.5280), // Continuation
    ];

    // When: Detect OBs
    let obs = OrderBlockDetector::detect(&candles);

    // Then: Bullish OB at index 0
    assert_eq!(obs.len(), 1);
    assert_eq!(obs[0].direction, OrderBlockDirection::Bullish);
    assert_eq!(obs[0].candle_index, 0);
    assert_eq!(obs[0].low, dec!(1.4900));
    assert_eq!(obs[0].high, dec!(1.5100));
}

#[test]
fn fvg_bullish_gap_detected() {
    // Given: Bearish candle followed by gap-up
    let candles = vec![
        candle!(high: 1.5000, low: 1.4900), // Previous bearish candle
        candle!(high: 1.5200, low: 1.5050), // Gap up (low > prev high)
    ];

    // When: Detect FVG
    let fvgs = FVGDetector::detect(&candles);

    // Then: Bullish FVG
    assert_eq!(fvgs.len(), 1);
    assert_eq!(fvgs[0].direction, FVGDirection::Bullish);
    assert_eq!(fvgs[0].top, dec!(1.5050));
    assert_eq!(fvgs[0].bottom, dec!(1.5000));
}
```

#### Confluence Engine Tests

```rust
#[test]
fn confluence_score_weights_sum_to_100() {
    // Given: Full alignment scenario
    let factors = vec![
        ConfluenceFactor::new(FactorType::HTFAlignment, 1.0),
        ConfluenceFactor::new(FactorType::Regime, 0.9),
        ConfluenceFactor::new(FactorType::OrderBlock, 0.8),
    ];

    // When: Calculate score
    let score = ConfluenceEngine::calculate(&factors);

    // Then: Weighted sum computed correctly
    let expected = (
        1.0 * HTF_ALIGNMENT_WEIGHT +
        0.9 * REGIME_WEIGHT +
        0.8 * OB_WEIGHT
    ) * 100.0;

    assert!((score.total as f64 - expected).abs() < 0.01);
}
```

**Coverage Target:** >80% line coverage per module

---

### 2. Property-Based Tests

**Goal:** Verify invariants hold across random inputs.

```rust
proptest! {
    #[test]
    fn confluence_score_always_0_to_100(factors in vec(any_factor(), 1..20)) {
        let score = ConfluenceEngine::calculate(&factors);
        prop_assert!(score.total <= 100);
        prop_assert!(score.total >= 0);
    }

    #[test]
    fn swing_high_always_above_surrounding(
        idx in 3..100usize,
        candles in vec(any_candle(), 10..200)
    ) {
        let swings = SwingDetector::detect(&candles, 3);

        for high in &swings.highs {
            if high.index > 0 && high.index < candles.len() - 1 {
                prop_assert!(high.price >= candles[high.index - 1].high);
                prop_assert!(high.price >= candles[high.index + 1].high);
            }
        }
    }

    #[test]
    fn bos_requires_structure_context(
        candles in vec(any_candle(), 5..50),
        direction in any_trend_direction()
    ) {
        // BOS cannot occur without prior structure
        let context = MarketContext::new(direction);
        let bos = BosDetector::detect(&candles, &context);

        if context.swing_points.len() < 2 {
            prop_assert!(bos.is_none());
        }
    }

    #[test]
    fn signal_determinism(
        symbol in any_symbol(),
        candle_data in vec(any_candle(), 100..500)
    ) {
        // Same input must produce same output
        let result1 = SignalEngine::detect(&symbol, &candle_data);
        let result2 = SignalEngine::detect(&symbol, &candle_data);

        prop_assert_eq!(
            result1.signals.len(),
            result2.signals.len()
        );

        for (s1, s2) in result1.signals.iter().zip(result2.signals.iter()) {
            prop_assert_eq!(s1.signal_id, s2.signal_id);
            prop_assert_eq!(s1.direction, s2.direction);
            prop_approx_eq!(s1.confluence_score.total, s2.confluence_score.total, 0.001);
        }
    }
}
```

**Invariants Tested:**
- Confluence scores always in range [0, 100]
- Swing highs/lows always extrema
- BOS/CHoCH requires prior structure
- Determinism: Same input → Same output
- No signal has negative R:R
- All timestamps monotonic

---

### 3. Integration Tests

**Goal:** Verify complete pipeline works end-to-end.

```rust
#[tokio::test]
async fn full_pipeline_generates_signal() {
    // Given: Market data from file
    let data = load_fixture("eurusd_uptrend.json");

    // When: Process through full pipeline
    let engine = SignalEngine::new(test_config()).await;
    let result = engine.process(&data).await;

    // Then: Signal generated with expected properties
    assert_eq!(result.signals.len(), 1);
    assert_eq!(result.signals[0].direction, TradeSide::Buy);
    assert!(result.signals[0].confluence_score.total >= 70);
}

#[tokio::test]
async fn mtf_alignment_blocks_wrong_direction() {
    // Given: HTF bearish, LTF bullish setup
    let htf_context = MarketContext::bearish_downtrend();
    let m15_data = bullish_order_block_setup(); // Would normally be buy signal

    // When: Process with HTF context
    let engine = SignalEngine::new(test_config()).await;
    engine.update_htf_context("EURUSD", htf_context);
    let result = engine.process(&m15_data).await;

    // Then: No buy signal (blocked by HTF)
    let buy_signals: Vec<_> = result.signals
        .iter()
        .filter(|s| s.direction == TradeSide::Buy)
        .collect();
    assert!(buy_signals.is_empty());
}

#[tokio::test]
async fn quality_filter_rejects_b_signals() {
    // Given: Valid B-quality signal setup
    let data = load_fixture("marginal_setup.json");
    let mut config = test_config();
    config.min_quality = SignalQuality::A;

    // When: Process with A minimum
    let engine = SignalEngine::new(config).await;
    let result = engine.process(&data).await;

    // Then: No signals emitted
    assert!(result.signals.is_empty());
}
```

---

### 4. Historical Replay Tests

**Goal:** Validate against known historical patterns.

#### Test Dataset

| Dataset | Period | Symbols | Timeframes | Signals Expected |
|---------|--------|---------|------------|------------------|
| 2024_Q1 | Jan-Mar 2024 | EURUSD, GBPUSD, XAUUSD | H4, H1, M30, M15 | ~450 |
| Trend_Forms | 20 selected trends | 5 majors | All | 20 |
| Ranges | 15 range periods | 5 majors | All | 0 (no signals in chop) |
| Reversals | 30 reversal points | 5 majors | All | 30 |

```rust
#[test]
fn historical_replay_eurusd_q1_2024() {
    // Load historical data
    let data = load_historical("eurusd_2024_q1.csv");

    // Run signal engine
    let mut engine = SignalEngine::new(production_config());
    let generated_signals: Vec<Signal> = vec![];

    for chunk in data.chunks(100) {
        let result = engine.process_chunk(chunk);
        generated_signals.extend(result.signals);
    }

    // Compare to golden dataset
    let expected = load_golden_signals("eurusd_2024_q1_signals.json");

    // Validate
    assert_signal_overlap(&generated_signals, &expected, 0.95); // 95% match
}

#[test]
fn range_market_produces_no_signals() {
    // Load known ranging period
    let data = load_historical("choppy_period.csv");

    let signals = run_engine(&data);

    // Should not overtrade in chop
    assert!(signals.len() <= 2); // Max 2 marginal signals
}
```

---

### 5. Determinism Tests

**Goal:** Verify identical inputs produce identical outputs.

```rust
#[test]
fn determinism_multi_run() {
    let input = load_fixture("complex_scenario.json");
    let config = production_config();

    let results: Vec<_> = (0..100)
        .map(|_| {
            let mut engine = SignalEngine::new(config.clone());
            engine.process(&input)
        })
        .collect();

    // All results identical
    let first = &results[0];
    for result in &results[1..] {
        assert_eq!(result.signals.len(), first.signals.len());
        for (s1, s2) in result.signals.iter().zip(first.signals.iter()) {
            assert_eq!(s1.signal_id, s2.signal_id);
            assert_eq!(s1.detected_at, s2.detected_at);
            assert_eq!(s1.confluence_score.total, s2.confluence_score.total);
            assert_eq!(s1.patterns_detected.len(), s2.patterns_detected.len());
        }
    }
}

#[test]
fn determinism_cross_platform() {
    // Load pre-computed outputs from Linux/x64
    let expected = load_fixture("linux_x64_output.json");
    let input = load_fixture("cross_platform_input.json");

    let result = SignalEngine::new(production_config()).process(&input);

    // Byte-for-byte identical
    assert_eq!(
        serde_json::to_string(&result).unwrap(),
        serde_json::to_string(&expected).unwrap()
    );
}
```

---

### 6. Performance Benchmarks

**Goal:** Establish and maintain performance baselines.

```rust
fn benchmark_signal_generation(c: &mut Criterion) {
    let data = load_fixture("eurusd_1000_candles.json");
    let engine = SignalEngine::new(production_config());

    c.bench_function("signal_generation_1000_candles", |b| {
        b.iter(|| {
            let mut e = engine.clone();
            e.process(black_box(&data))
        });
    });
}

fn benchmark_mtf_alignment(c: &mut Criterion) {
    let htf_data = load_fixture("htf_context.json");
    let ltf_data = load_fixture("ltf_50_candles.json");

    c.bench_function("mtf_alignment_4_timeframes", |b| {
        b.iter(|| {
            MTFAligner::align(
                black_box(&htf_data),
                black_box(&ltf_data),
                Timeframe::M15
            )
        });
    });
}

fn benchmark_confluence_calculation(c: &mut Criterion) {
    let patterns = generate_test_patterns(10);
    let context = MarketContext::example();

    c.bench_function("confluence_10_factors", |b| {
        b.iter(|| {
            ConfluenceEngine::calculate_full(
                black_box(&patterns),
                black_box(&context)
            )
        });
    });
}

criterion_group!(
    benches,
    benchmark_signal_generation,
    benchmark_mtf_alignment,
    benchmark_confluence_calculation
);
criterion_main!(benches);
```

**Performance Targets:**

| Metric | Target | Worst Case |
|--------|--------|------------|
| Signal latency (P99) | < 10ms | < 50ms |
| Candles/sec (single symbol) | 1000+ | 500+ |
| Memory per symbol | < 5MB | < 10MB |
| CPU usage (sustained) | < 10% | < 25% |

---

### 7. Golden Dataset Tests

**Goal:** Validate against known-good signal sets.

```rust
/// Golden signals are manually annotated by senior traders
/// as "this definitely should have generated a signal"
#[test]
fn golden_signals_all_detected() {
    let golden = load_golden_dataset("v1_golden_signals.json");

    let mut detected_count = 0;
    let mut false_negatives = vec![];

    for case in golden.cases {
        let result = SignalEngine::new(production_config())
            .process(&case.candles);

        let found = result.signals.iter().any(|s| {
            s.direction == case.expected_direction &&
            (s.detected_at - case.expected_time).abs() < Duration::minutes(5)
        });

        if found {
            detected_count += 1;
        } else {
            false_negatives.push(case);
        }
    }

    let detection_rate = detected_count as f64 / golden.cases.len() as f64;
    assert!(
        detection_rate >= 0.95,
        "Detection rate {} below 95%. False negatives: {:?}",
        detection_rate,
        false_negatives
    );
}

#[test]
fn golden_non_signals_not_triggered() {
    // Cases where signal should NOT fire
    let non_signals = load_golden_dataset("v1_non_signals.json");

    for case in non_signals.cases {
        let result = SignalEngine::new(production_config())
            .process(&case.candles);

        assert!(
            result.signals.is_empty(),
            "False positive on case {}: {:?}",
            case.id,
            result.signals
        );
    }
}
```

---

## Test Fixtures

### Fixture Format

```json
{
  "name": "eurusd_downtrend_bos",
  "description": "Clear downtrend with BOS continuation",
  "symbol": "EURUSD",
  "timeframes": {
    "H4": { "bias": "bearish", "structure": "downtrend" },
    "H1": { "bias": "bearish", "last_choch": null },
    "M30": { "bias": "bearish", "last_bos": "2024-01-15T10:00:00Z" }
  },
  "candles": [
    {"time": "2024-01-15T10:00:00Z", "open": 1.0950, "high": 1.0960, "low": 1.0940, "close": 1.0945, "volume": 1000},
    // ... 200 more candles
  ],
  "expected": {
    "signal_count": 1,
    "signals": [
      {
        "direction": "sell",
        "pattern_type": "bos_continuation",
        "confluence_min": 75,
        "confluence_max": 90,
        "entry_zone": {"low": 1.0920, "high": 1.0930}
      }
    ]
  }
}
```

### Fixture Categories

| Category | Count | Purpose |
|----------|-------|---------|
| SMC Patterns | 50 | Test each pattern type |
| Trend Continuation | 30 | BOS in direction of trend |
| Trend Reversal | 30 | CHoCH against trend |
| Ranging Markets | 20 | Low quality, should reject |
| HTF Alignment | 20 | Test MTF logic |
| Edge Cases | 20 | Gaps, low liquidity, etc |

---

## CI/CD Integration

```yaml
# .github/workflows/signal-engine-tests.yml
name: Signal Engine Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run unit tests
        run: cargo test --lib -- --test-threads=4

      - name: Run property tests
        run: cargo test --features proptest -- proptest

      - name: Run integration tests
        run: cargo test --test '*'
        env:
          TEST_REDIS_URL: redis://localhost:6379

      - name: Run historical replay
        run: cargo test --test historical -- --nocapture

      - name: Run benchmarks
        run: cargo bench -- --test

      - name: Check coverage
        run: |
          cargo tarpaulin --out Xml
          bash <(curl -s https://codecov.io/bash)
```

---

## Test Success Criteria

| Category | Minimum | Target |
|----------|---------|--------|
| Unit test coverage | 80% | 90% |
| Integration test pass | 100% | 100% |
| Historical replay match | 95% | 98% |
| Determinism | 100% | 100% |
| Performance targets | 100% | 100% |
| Golden detection rate | 95% | 98% |
| False positive rate | <5% | <2% |

---

*Document Version: 1.0*  
*Last Updated: 2026-06-14*
