# Intelligence Implementation

The Intelligence Engine (`src/intelligence/mod.rs`) is the core decision-maker for the Strategy Engine. It is responsible for assessing edges and generating deterministic recommendations without executing trades.

## Key Components

### PatternAssessment
Categorizes patterns into `Exceptional`, `Strong`, `Normal`, `Weak`, and `Negative`. Based exclusively on `expectancy` and `stability` calculations.

### EdgeIntelligence
Holds core metrics: `expectancy`, `win_rate`, `rr`, `stability`, `drawdown`.
Provides:
- `assess()`: Maps the numerical edge data into a `PatternAssessment`.
- `recommend()`: Maps the `PatternAssessment` into a deterministic `Recommendation`.

### ExpectancyAssessment
Tracks quality, degradation, and acceleration of expectancy over time.

## Guarantees
- 100% deterministic (no randomness, no floating point arithmetic).
- Utilizes `rust_decimal::Decimal` exclusively.
- Zero-panic guarantees.
