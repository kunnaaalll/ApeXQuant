# STRESS ENGINE IMPLEMENTATION

## Overview

The Stress Engine is Phase 8 of the APEX V3 Risk Engine. It is a deterministic, high-performance simulation subsystem designed to answer one crucial question: Can the portfolio survive extreme market events?

## Key Constraints
- **Zero Floating Point Arithmetic:** All core logic uses `rust_decimal::Decimal`.
- **Zero Panic:** Use of `panic!`, `unwrap()`, or `expect()` is forbidden.
- **Zero Randomness:** Tests must use predefined deterministic scenarios and replayable snapshots.
- **100% Determinism:** Multiple iterations of identical inputs guarantee identical outputs without precision drift.

## Modules

- `scenarios.rs`: Library of historical and extreme events.
- `volatility.rs`: Models volatility shocks avoiding negative thresholds.
- `correlation.rs`: Simulates asset correlations collapsing toward 1.0.
- `liquidity.rs`: Estimates spread and slippage behavior during crises.
- `leverage.rs`: Models cascading amplification resulting from embedded gross/hidden leverage.
- `survival.rs`: Computes a robust score mapping to `SurvivalState` states.
