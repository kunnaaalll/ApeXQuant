# Market Intelligence Engine (Phase 4)

The Phase 4 Market Intelligence Engine transforms raw market data into deterministic institutional intelligence for consumption by downstream engines (strategy, risk, portfolio, execution, ai).

## Architecture

* **Zero-Float Determinism**: All calculations use `rust_decimal::Decimal` and bounded integers (`u8`, `u32`, `i64`). No `f32` or `f64` types are permitted to guarantee deterministic execution across all environments.
* **Panics and Unsafe Code**: `#![deny(unsafe_code)]` is strictly enforced. The engine uses zero `unwrap()`, `expect()`, or panics. All state transitions use standard `Result` types for error handling.
* **Component-Based Pipelines**: Intelligence metrics are calculated in parallel via modular sub-engines (Volatility, Trend, Momentum, Structure, Correlation, Regime, Quality) and aggregated deterministically into a single `MarketIntelligenceProfile`.
* **Validation**: The `ValidationFramework` acts as an invariant check on every aggregated profile.

## Usage

Instantiate sub-engines with required periods, sequentially call `.update()` functions on each new tick, and pass the results to `IntelligenceAggregator::build_profile()` to build the unified `MarketIntelligenceProfile`.
