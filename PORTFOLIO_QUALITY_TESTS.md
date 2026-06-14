# APEX V3 — Portfolio Quality Tests

## Testing Strategy
The Quality engine prioritizes deterministic behavior under complex portfolio conditions. It undergoes rigorous property and unit testing.

## Unit Tests
- `test_quality_initialization`: Validates default neutral behavior with base values.
- `test_quality_state_boundaries`: Ensures bounds mapped precisely.
    - `Excellent` maps strictly >= 90.0.
    - `Good` maps strictly >= 75.0 and < 90.0.
    - `Neutral` maps strictly >= 50.0 and < 75.0.
    - `Weak` maps strictly >= 25.0 and < 50.0.
    - `Critical` maps < 25.0.
- `test_apply_event_clamping`: Ensures external input values exceeding [0.0, 100.0] are strictly clamped.
- `test_apply_decay`: Validates score degradation rules properly step down state mappings sequentially.

## Property Tests
- **Score Bound Validity**: Fuzz testing confirms composite scores never breach 0.0 or 100.0.
- **Timestamp Monotonicity**: Verifies that subsequent versions of `QualitySnapshot` never have a lower timestamp.

## Replay and Stress Tests
- Simulating sustained drawdown phases to test proper propagation of decay and eventual state changes.
- Performance profiling targets zero allocations in the hot path.
