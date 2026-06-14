# Adaptive Optimization Implementation

## Overview
The `adaptive` module implements deterministic learning using exponential moving averages (EMA) without the use of floating-point arithmetic. It bounds optimizations to prevent runaway edge chasing.

## Core Modules
- `decay_model.rs`: Calculates EMA using `rust_decimal` to prevent precision loss. It strictly bounds `alpha` between `(0.0, 1.0]`.
- `score_adjuster.rs`: Maps unbounded raw values (like expectancy) into a standard 0 to 100 confidence score to be used by weight adjusters.
- `weight_optimizer.rs`: Adjusts the allocation weight of a given entity (symbol, pattern, regime) safely towards a target, clamped by `min_weight` and `max_weight`, and moving at most `max_step` per period.

## Mathematical Determinism Guarantee
All operations use `rust_decimal::Decimal`. The use of `f32`/`f64` is prohibited, ensuring that processing the identical sequence of inputs across different architectures will yield a byte-for-byte identical internal state.
