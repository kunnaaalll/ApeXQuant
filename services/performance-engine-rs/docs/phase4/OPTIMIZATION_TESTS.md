# Optimization Tests Strategy

## Overview
The adaptive engine must adhere to strict determinism and high performance. It must run 100,000 optimization iterations without memory leaks, panics, or divergence.

## Testing Layers

### 1. Unit Tests
- **Verification:** Mathematical correctness of the `DecayModel` and `WeightOptimizer`.
- **Coverage:** Boundaries testing minimum limits (`0.5`), maximum limits (`2.0`), and step enforcement (`0.05`).

### 2. Property Tests
- **Verification:** Ensures functions mapping unbounded inputs (expectancy, drawdown) to explicit states (`Elite`, `Forbidden`, `Collapse`) are fully deterministic.
- **Tools:** Use `proptest` to generate extreme `rust_decimal` values, asserting `#[deny(unsafe_code)]` compliance and zero division-by-zero panics.

### 3. Replay & Determinism Tests
- **Verification:** Emulate 100,000 simulated input vectors and assert byte-for-byte equality across different compilation profiles and operating systems.
- **Divergence:** Strict checking that varying historical pathing but equal total expectancy produces identical decayed outputs given identical event structures.

### 4. Stress & Latency Tests
- **Verification:** Measure latency through hot paths.
- **Constraints:** P99 execution under 10ms and average under 2ms. Asserts zero lock contention and zero unexpected allocations.
