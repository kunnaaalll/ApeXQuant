# Risk Validation Invariants

1. **Zero Randomness**: The engine must never use random behavior during validation or runtime. All logic is strictly deterministic.
2. **Zero Floats**: No `f32` or `f64` types. All arithmetic utilizes `rust_decimal` or integer types to guarantee reproducible financial results.
3. **Zero Panics**: The `unwrap`, `expect`, and `panic` macros are forbidden. The engine must safely propagate all errors.
4. **Zero Unsafe**: `#![deny(unsafe_code)]` is strictly enforced.
5. **No State Drift**: 100,000 iterations over the same event stream must yield byte-for-byte exact state matches.
