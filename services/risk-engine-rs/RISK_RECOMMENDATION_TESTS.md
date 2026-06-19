# Risk Recommendation Tests

## Test Coverage Requirements

1. **Frozen Always Dominates**: (`test_frozen_always_dominates`)
   - Asserts `DrawdownState::Frozen` overrides all other healthy inputs.

2. **Emergency Priority**: (`test_emergency_reduction_priority`)
   - Asserts a `TailRiskScore::Collapse` always maps to an emergency reduction and restricts trades.

3. **Inconsistent States**: (`test_inconsistent_states_rejected`)
   - Asserts `validate_consistency()` catches `IncreaseRisk` + `Freeze` trade admission policies.

4. **100k Iterations**: (`test_recommendation_determinism_100k`)
   - Tests engine outputs 100,000 times to guarantee absolute determinism and performance.

5. **Rebuild Snapshot**: (`test_snapshot_rebuild`)
   - Reconstructs a state snapshot entirely from saved variables.
