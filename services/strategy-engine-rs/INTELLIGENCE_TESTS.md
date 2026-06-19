# Intelligence Tests

Comprehensive testing logic covering the complete scope of Phase 3 elements.

## Testing Layers

### Unit Tests
Validates precise transitions and exact `rust_decimal::Decimal` calculations.
- `intelligence.rs`: Verifies PatternAssessment boundaries and recommendation transitions.
- `confidence.rs`: Verifies penalty accumulation and strict clamping on ConfidenceScore calculations.
- `streaks.rs`: Verifies recovery dampening and streak categorization.
- `drift.rs`: Verifies boundary limits (-20%, -40%, -60%).
- `learning.rs`: Verifies EMA accumulation rules.
- `memory.rs`: Verifies bounded `VecDeque` retention and ejection.

### Determinism Tests
Located in `determinism_tests.rs`.
Executes loops of up to 100,000 iterations to absolutely confirm that compounding calculations (like Learning EMA) do not drift uncontrollably into non-deterministic states.

### Panic Enforcement
Guaranteed explicitly by `#![deny(clippy::unwrap_used)]`, `#![deny(clippy::expect_used)]`, and `#![deny(clippy::panic)]`. Verified automatically by our cargo check cycles.
