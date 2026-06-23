# Failover Engine

This document outlines the determinism requirements and architecture for the Failover Engine module in APEX V3 Phase 2.

## Constraints
- No floating-point types (`f32`/`f64`).
- No `unsafe` code.
- No `unwrap()` or `expect()`.
- Zero side effects.
