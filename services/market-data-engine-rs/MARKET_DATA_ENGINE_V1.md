# Market Data Engine V1

This document outlines the determinism requirements and architecture for the Market Data Engine V1 module in APEX V3.

## Constraints
- No floating-point types (`f32`/`f64`).
- No `unsafe` code.
- No `unwrap()` or `expect()`.
- Zero side effects.
