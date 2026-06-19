# APEX V3 Strategy Engine V1

The central deterministic state machine governing the transition, evaluation, and lifecycle of all algorithmic strategies within the APEX ecosystem.

## Key Principles
- **100% Deterministic**: All numerical calculations use `rust_decimal::Decimal`.
- **Zero Panic Guarantee**: `#![deny(unsafe_code)]` alongside strict clippy linting for zero `unwrap`, `expect`, or `panic`.
- **Replayable State**: Every state transition emits immutable domain events.
