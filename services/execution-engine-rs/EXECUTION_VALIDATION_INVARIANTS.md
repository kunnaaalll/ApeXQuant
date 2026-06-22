# Execution Validation Invariants

- `#![deny(unsafe_code)]`
- zero unwrap
- zero expect
- zero panic
- zero randomness
- zero floats (all math uses `rust_decimal::Decimal`)
- zero side effects
- deterministic outputs only
