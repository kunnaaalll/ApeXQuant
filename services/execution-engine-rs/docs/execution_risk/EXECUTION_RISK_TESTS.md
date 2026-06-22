# Execution Risk Tests

The test suite validates the strict invariants of the execution layer.

## Key Tests
- `test_state_transition_rules`: Verifies illegal state jumps are rejected.
- `test_spread_bounds`, `test_latency_bounds`: Verifies all scores map bounded to `0-100`.
- `test_rejection_lock`: Ensures consecutive rejections result in Frozen states.
- `test_determinism_100k_iterations`: 100,000 loop execution validating identical outputs, zero drift, zero panics, and zero randomness under load.
