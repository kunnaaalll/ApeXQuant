# Optimization Invariants

## Core Principles

The adaptive intelligence layer of APEX V3 relies upon strict mathematical invariants to ensure zero panic, determinism, and institutional readiness.

1. **Weight Boundaries:**
   - No entity (symbol, pattern, regime) weight can ever be optimized to a value outside the absolute bounds of `[0.5, 2.0]`. 
   - A single-step weight adjustment can never exceed `0.05` under any market condition.

2. **Decay Mechanics:**
   - The smoothing factor (`alpha`) must reside strictly in the domain `(0.0, 1.0]`.
   - Exponential Moving Average calculations cannot overflow; bounded context constraints dictate maximum acceptable raw values.

3. **Absolute Confidence Constraints:**
   - Confidence scoring is strictly normalized to the `[0.0, 100.0]` range.
   - Any raw expectancy evaluation lacking a minimum required sample size defaults mathematically to a neutral confidence state and prohibits any upward weight optimization.

4. **Float Extermination:**
   - The engine utilizes exactly 0 instances of `f32` or `f64`.

These invariants form the basis of the optimization and discovery test suite.
