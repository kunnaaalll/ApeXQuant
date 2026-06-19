# Drift Engine Implementation

The Drift Engine (`src/drift/mod.rs`) detects macro structural drift across several strategy dimensions, serving as an early-warning system for strategy degradation.

## Components

### DriftEngine
Tracks four dimensions of drift:
- `edge_drift`
- `expectancy_drift`
- `confidence_drift`
- `stability_drift`

### DriftState
Generates discrete categorical states (`Improving`, `Stable`, `Weakening`, `Critical`, `Collapse`) based on exact decimal thresholds (-20%, -40%, -60%, +10%). State evaluation uses min and max boundary evaluations rather than complex aggregations to ensure precision and determinism.

## Guarantees
- 100% deterministic (no randomness, no floating point arithmetic).
- Utilizes `rust_decimal::Decimal` exclusively.
- Zero-panic guarantees.
