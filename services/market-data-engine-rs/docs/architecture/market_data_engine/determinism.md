# Determinism Guarantee

The Market Data Engine guarantees bit-for-bit exact reproducibility across 100,000 iterations for institutional grade reliability.

## Invariants

- **Zero-Float Types**: `f32` and `f64` are banned to avoid non-deterministic hardware rounding. Instead, `rust_decimal::Decimal` is exclusively used.
- **Bounded Mathematics**: Intermediate aggregates limit size to bounded integer ranges (e.g. `u8`, `u16`, `u32`).
- **No Unsafe Code**: Standard library safe Rust exclusively.
- **No Exceptions**: Unwrapping or explicit panicking will halt compilation (enforced via `deny(clippy::unwrap_used)`). All methods return `Result<T, E>`.
- **Test Framework Verification**: 100k iteration test harnesses are used to verify deterministic looping across all edge cases without divergence or panic.
