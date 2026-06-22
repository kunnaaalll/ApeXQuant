# Execution Shadow Mode Invariants

The Shadow Mode is guarded by strictly enforced compilation and runtime rules.

## Compilation 

```rust
#![deny(unsafe_code)]
```
Memory must be absolute.

## Execution Rules

1. **Zero Panics:** Expected values handle Edge/Corner cases defensively. Division-by-zero maps to extreme bounds rather than panic drops.
2. **Zero Floats:** `rust_decimal::Decimal` is absolute. IEEE 754 precision loss is rejected immediately. 
3. **Zero Side-Effects:** The Comparison, Drift, Health and Validate logic are pure math functions. No I/O is touched.
4. **Zero Broker Mutations:** Execution outputs are logged and reported, never pushed to live network routers.
