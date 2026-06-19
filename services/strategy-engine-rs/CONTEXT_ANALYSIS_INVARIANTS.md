# Context Analysis Invariants

## Zero Panics
No occurrences of `panic!`, `unwrap()`, or `expect()` are permitted within any Context Intelligence module.

## Determinism
All score compounding is performed via `rust_decimal`.

## Immutability
All analytical engines operate purely on `&self` taking discrete state inputs without internal mutability to ensure replayability.
