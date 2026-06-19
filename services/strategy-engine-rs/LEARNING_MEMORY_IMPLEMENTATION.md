# Learning & Memory Implementation

These engines (`src/learning/mod.rs` and `src/memory/mod.rs`) provide stateful, bounded historical context without resorting to AI, ML, or non-deterministic data structures.

## Learning Engine
### EvidenceAccumulator
Aggregates historical data over time. Implements an Exponential Moving Average (EMA) explicitly using fixed decimal math rather than floats to ensure `rust_decimal::Decimal` precision and strictly replayable outputs. 
Yields discrete `LearningAssessment` recommendations based on the EMA values.

## Memory Engine
Maintains explicit history up to a bounded capacity, bypassing infinite allocation concerns and guaranteeing predictable performance.
### ConfidenceMemory
Tracks bounded `VecDeque` representations of historical confidence, degradation, and edge tracking.
### EdgeMemory
Maintains a rolling bounded history of pure edge and expectancy.

## Guarantees
- Bounded, fixed capacities to ensure predictable allocation.
- 100% deterministic (no randomness, no floating point arithmetic).
- Utilizes `rust_decimal::Decimal` exclusively.
- Zero-panic guarantees.
