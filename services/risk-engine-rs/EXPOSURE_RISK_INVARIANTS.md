# Exposure Risk Engine Invariants

The exposure risk engine is designed to operate in high-reliability institutional environments. As such, it guarantees the following invariants at compile and runtime:

## Safety Invariants
- `#![deny(unsafe_code)]` is strictly enforced. No FFI or raw pointer arithmetic is permitted within the exposure module.
- **Zero Panics**: The use of `.unwrap()`, `.expect()`, and `panic!()` is strictly denied via clippy lint rules `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
- **Numerical Safety**: Float arithmetic is entirely removed. All calculations utilize `rust_decimal::Decimal` which handles arbitrary precision base-10 mathematics deterministically, avoiding `NaN` or `Infinity`.

## Domain Invariants
- `Gross Exposure >= abs(Net Exposure)`
- `Concentration Score ∈ [0, 100]`
- `Diversification Score ∈ [0, 100]`
- Exposures (Gross, Long, Short) are strictly tracked in absolute magnitude or explicitly signed depending on the struct, but total Gross is always `>= 0`.

## Architecture Invariants
- **Immutable Events**: `ExposureRiskEvent` variants contain no internal mutability cells (`RefCell`, `Mutex`).
- **State Latches**: Once the engine enters a `Frozen` state, it cannot deterministically revert to `Normal` through standard exposure changes without an explicit manual override or resolution.
