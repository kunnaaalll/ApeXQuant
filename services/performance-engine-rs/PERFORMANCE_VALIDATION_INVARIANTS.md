# Performance Validation Invariants

## Core Invariants
All modifications to the performance engine MUST respect the following invariants. Any breach during validation testing results in an immediate downgrade from `InstitutionalCertified` state.

1. **Determinism:** Given inputs `E_1..E_n`, the output `State(E_n)` must remain identical across any number of replays. Zero deviation is permitted.
2. **Replay Integrity:** `State(E_n)` computed sequentially via streaming must exactly equal `State(E_n)` computed via snapshot rebuild.
3. **Immutability of Snapshots:** Snapshots, once written, cannot be mutated.
4. **Zero Floating Point Errors:** All logic MUST execute through `rust_decimal`. Use of `f32` or `f64` in mathematical core logic is strictly forbidden.
5. **No Panics:** Malformed inputs or missing contexts must gracefully return a degraded state or `Result::Err`, never unwrapping a `None`.
6. **Thread Safety:** The engine structure must remain `Send + Sync` to permit massive parallel Monte Carlo simulations and event bursts without deadlocks.
