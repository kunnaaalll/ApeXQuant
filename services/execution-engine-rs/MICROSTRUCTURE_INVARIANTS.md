# MICROSTRUCTURE INVARIANTS

## Strict Core Guarantees
1. `#![deny(unsafe_code)]` at the crate boundary.
2. Absolutely no floating-point logic (`f32`, `f64`). Standardized entirely on `rust_decimal::Decimal`.
3. Panic-free execution. No `unwrap()` or `expect()` calls allowed within analytical calculations.
4. Bound invariants (0-100 scores) guaranteed internally by safe clamping and strict bounds checking.
5. Absolute determinism across iterative loops and replays.
