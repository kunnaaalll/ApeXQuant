# Streaming Invariants

This document outlines the determinism requirements and architecture for the Streaming Invariants module in APEX V3 Phase 3.

## Constraints
- No floating-point types (`f32`/`f64`).
- No `unsafe` code.
- No `unwrap()` or `expect()`.
- Zero side effects.
