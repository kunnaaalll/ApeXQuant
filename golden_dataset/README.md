# APEX Golden Dataset

This directory contains validation-quality historical market data for testing the Signal Engine.

## Structure

```
golden_dataset/
├── README.md           # This file
├── metadata.json       # Dataset metadata
├── scenarios/          # Individual test scenarios
│   ├── 0001_uptrend_eurusd_m15.json
│   ├── 0002_downtrend_gbpusd_h1.json
│   └── ...
└── fixtures/           # Shared test data
    ├── candles/
    └── expected/
```

## Scenario Categories

### Trending Scenarios
- **TrendingUp**: Strong bull trend with HH/HL structure
- **TrendingDown**: Strong bear trend with LL/LH structure

### Range Scenarios
- **Ranging**: Sideways market with clear support/resistance
- **LowVolatility**: Contracting range, low ATR
- **HighVolatility**: Expanding range, high ATR

### Breakout Scenarios
- **Breakout**: Clean break of key level
- **LiquiditySweep**: Stop hunt followed by reversal

### SMC Pattern Scenarios
- **StrongBos**: Clear break of structure
- **StrongChoch**: Clear change of character
- **GoodOrderBlock**: Valid unmitigated OB
- **GoodFvg**: Valid fair value gap

### Complex Scenarios
- **Complex**: Multiple patterns, overlapping signals

## Format

Each scenario is a JSON file with:

```json
{
  "scenario_id": "unique-id",
  "name": "Human-readable name",
  "description": "What's being tested",
  "category": "TrendingUp",
  "symbol": "EURUSD",
  "timeframe": "M15",
  "start_time": "2024-01-01T00:00:00Z",
  "end_time": "2024-01-01T04:00:00Z",
  "candles": [...],
  "expected_signals": [...]
}
```

## Usage

```rust
use signal_engine::replay::{DatasetLoader, GoldenDataset};

let loader = DatasetLoader::new();
let dataset = loader.load_from_directory(Path::new("golden_dataset"))?;

// Run all scenarios
for scenario in &dataset.scenarios {
    let result = replay_engine.run_determinism_test(scenario, 100).await?;
    assert!(result.is_deterministic);
}
```

## Validation Criteria

Each scenario should validate:
1. **Determinism**: Same output every run (100 iterations)
2. **Latency**: <10ms average, <25ms P99
3. **Parity**: Direction matches TypeScript reference
4. **Explainability**: Evidence provided for all signals

## Go-Live Criteria

Dataset must demonstrate:
- 95%+ direction agreement with TypeScript
- <10% confidence drift
- 90%+ pattern agreement
- 90%+ regime agreement
- Zero panics across all scenarios
