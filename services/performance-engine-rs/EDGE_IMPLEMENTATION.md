# Edge Module Implementation

## Overview
The Edge module quantifies the raw mathematical advantage present in the strategy. Unlike predictive systems, this engine measures realized edge deterministically based on empirical results.

## Core Metrics
- **Raw Edge:** Unbounded raw measure of statistical advantage over random entry/exit.
- **Edge Score:** Normalized 0-100 score representing edge quality.
- **Edge Acceleration:** Rate of change of the edge (positive indicates improving edge).
- **Edge Decay:** Rate of loss of the edge.
- **Edge Confidence:** Sample-size adjusted confidence interval weighting.
- **Edge Stability:** Variance in the edge over sequential sub-periods.

## State Transitions
- **HighEdge:** Edge score > 80.
- **MediumEdge:** Edge score 50-80.
- **LowEdge:** Edge score 20-50.
- **NoEdge:** Edge score < 20.

## Safe Math Guarantees
- No usage of `f64`.
- All normalizations strictly clamped to 0-100.
- `rust_decimal` used for all operations.

## Events & Storage
- `EdgeEvent::Assessed(Assessment)`
- `EdgeSnapshot` for O(1) state reconstruction.
