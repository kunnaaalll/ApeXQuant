# Risk Storage Invariants

The storage engine relies on the following invariants. Breaching any of these invalidates the deterministic nature of the Risk Engine.

## Ordering
```text
sequence(n+1) > sequence(n)
```
- No gaps are permitted in an aggregate's event stream.
- No duplicates.
- The timestamp of `n+1` must be `>=` timestamp of `n`.

## Determinism
Replaying the same stream of events must ALWAYS yield the exact same `RiskState`.
- No floating point numbers (`f32`, `f64`) may be used (use `Decimal`).
- No external state or randomness (`rand`) during event application.

## Transactional Atomicity
- If an event and a snapshot are persisted simultaneously, they must commit via the same database transaction.

## Pure Functions
The `rebuild` logic must remain pure. It is strictly a `fold` operation over an array of `EventRecord`.
