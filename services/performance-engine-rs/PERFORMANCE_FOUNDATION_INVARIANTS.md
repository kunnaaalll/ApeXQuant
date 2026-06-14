# Performance Foundation Invariants

## Core Principles
1. **Zero Unsafe:** No `unsafe` blocks are allowed anywhere in the Performance Engine.
2. **Zero Panics:** The engine must never crash. Explicit `unwrap()` or `expect()` calls without guaranteed prior invariant checks are forbidden. Division by zero must yield safe fallback paths.
3. **Deterministic:** No usage of random numbers (`rand`), system time for calculation logic (UTC is for timestamping only), or `f64` (floating point math). Everything runs on `rust_decimal::Decimal`.
4. **Append-Only:** States are derived from events. Mutations are expressed as state transitions stored via events.

## Precision Guarantees
- `rust_decimal` maintains 28 significant digits.
- Rounding strategy must be consistent across all calculators (e.g. `HalfUp`).
- Bounded maximums: Infinite ratios are capped at specific boundaries (e.g., maximum profit factor 1000.0) to preserve mathematical continuity without breaking serializability.
