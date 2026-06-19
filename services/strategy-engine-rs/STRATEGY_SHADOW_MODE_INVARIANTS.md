# Shadow Mode Invariants

The APEX V3 Strategy Engine adheres to extreme constraints when running in shadow mode to ensure absolutely zero operational risk.

## Core Rules

1. `#![deny(unsafe_code)]`
2. **No floating point arithmetic** (`f32`/`f64`). All math must use `rust_decimal::Decimal`.
3. **No panic!(), expect(), or unwrap()**. Complete error handling and graceful fallbacks.
4. **No randomness or non-determinism**. 100,000 runs must produce identical results.
5. **No ML calls inside the core shadow loop**.
6. **Percentages must always be clamped** between 0 and 100.
7. **Divisions must never target zero**. Max(ref, 1) is strictly enforced.
