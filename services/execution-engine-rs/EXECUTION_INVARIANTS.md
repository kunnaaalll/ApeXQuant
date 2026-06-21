# Execution Invariants

1. **No Floating Point**: `f32` and `f64` are strictly banned. Everything uses `rust_decimal::Decimal`.
2. **Determinism**: No `rand` or RNG sources. All states are purely derived from explicit inputs.
3. **Replayability**: The `SmartExecutionSnapshot` reconstructed from a series of `SmartExecutionEvent`s must exactly match the state as it evolved live.
4. **No Panics**: No `unwrap()`, `expect()`, or `panic!` macros in production code. Use `Result`.
5. **Exact Quantities**: Split algorithms must preserve exact total quantity (rounding remainders go to the last slice).
