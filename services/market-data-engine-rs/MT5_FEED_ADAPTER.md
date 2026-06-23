# Mt5 Feed Adapter

This document outlines the determinism requirements and architecture for the Mt5 Feed Adapter module in APEX V3 Phase 2.

## Constraints
- No floating-point types (`f32`/`f64`).
- No `unsafe` code.
- No `unwrap()` or `expect()`.
- Zero side effects.
