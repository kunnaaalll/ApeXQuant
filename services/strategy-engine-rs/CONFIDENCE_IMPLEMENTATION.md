# Confidence Engine Implementation

The Confidence Engine (`src/confidence/mod.rs` and `src/confidence/penalty.rs`) calibrates institutional trust in an edge based on real-world stability, variance, and sample quality.

## Components

### ConfidenceScore
A strict `0-100` range metric mapped to `ConfidenceTier` (`VeryLow`, `Low`, `Normal`, `High`, `VeryHigh`).
Initialization explicitly clamps bounds to prevent overflow or out-of-range assignments.

### SampleQuality
Grades samples as `Insufficient`, `Weak`, `Adequate`, `Strong`, or `InstitutionalGrade`.
Thresholds strictly enforced at 20, 50, 100, 300, and 1000.

### ConfidencePenalty
Calculates deductions by evaluating drawdowns, variance, instability, edge decay, and consecutive losses.

## Guarantees
- 100% deterministic (no randomness, no floating point arithmetic).
- Utilizes `rust_decimal::Decimal` exclusively.
- Zero-panic guarantees.
