# Execution Risk Invariants

This module enforces the following strict engineering constraints:
- `#![deny(unsafe_code)]`
- Zero `panic!` calls.
- Zero `unwrap()` calls.
- Zero `expect()` calls.
- Zero randomness.
- Zero floating point (`f32`/`f64`) operations; exclusively uses `rust_decimal::Decimal`.
- Event sourced state transitions with exhaustive logging.
- All operations are completely deterministic.
