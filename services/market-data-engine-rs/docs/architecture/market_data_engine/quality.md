# Quality Engine

The Quality engine grades the real-time execution environment based on liquidity, spreads, sequence integrity, and feed health.

## Grade Calculation

- Checks spread ratio versus rolling average spread.
- Checks liquidity depth ratio versus rolling average.
- Grades sequence gaps and general boolean feed health.
- Calculates an aggregated `overall_score` out of 100.
- Assesses a `QualityGrade` (e.g. Elite, Excellent, Good, Average, Poor, Untradeable).

## Defensive Operations

- Immediately bails with a zero score or `Err` if average spreads are exactly zero (preventing panics).
- Handles division by zero gracefully using `rust_decimal`'s `.is_zero()` checks.
