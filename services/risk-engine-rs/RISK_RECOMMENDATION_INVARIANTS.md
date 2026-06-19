# Risk Recommendation Invariants

## Core Invariants

1. **Hierarchy of Action**: `Freeze > EmergencyReduction > Reduction > Maintain > Increase`. Higher severity states absolutely dominate.
2. **Explainability**: No recommendation may lack an explanation. (`test_explanation_has_no_empty_fields`).
3. **Impossible Combinations**: Banned states include `IncreaseRisk` combined with `FreezeTrading` or `Block`. They map to `ConsistencyError`.
4. **Append-only Events**: The risk committee only emits immutable events.
5. **No State Skipping**: Transitions between states must be methodical, no jumping from Collapse directly to Aggressive without intermediate normalization.
