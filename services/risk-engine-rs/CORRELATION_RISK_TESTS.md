# Correlation Risk Tests

To ensure the engine operates precisely under extreme conditions, it employs exhaustive property-based testing and deterministic checks.

## Key Scenarios

### Matrix Symmetry
A correlation from A to B must exactly equal B to A. We utilize `proptest` to fuzz matrices with varied string combinations and ensure equality to the last decimal place.

### Deterministic Precision Bounds
All values must precisely exist in `[-1.0, 1.0]`. If `Decimal::new(-2, 0)` is passed, it must clamp to `Decimal::new(-1, 0)`.

### Event Sourcing & Snapshots
`CorrelationRiskSnapshot` acts as a replay anchor. By generating synthetic `CorrelationRiskEvent` models, we ensure the matrix state, hidden leverage state, and clustering behavior can be identically replayed across any machine architecture.

### Zero Panics
Tested via large loop insertions and edge-case boundary checks to guarantee 100% panic-free performance.
