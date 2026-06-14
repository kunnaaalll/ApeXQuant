# Recommendation Engine Invariants

## Capital Preservation Rules
- **Rule 1**: The engine MUST prioritize capital preservation over growth.
- **Rule 2**: If the portfolio enters a `Frozen` state, all recommendations must default to maximal preservation:
  - Increase: `Reject`
  - Reduce: `EmergencyReduction`
  - Close: `EmergencyLiquidation`
  - Block: `Freeze`
- **Rule 3**: If the portfolio enters a `Critical Drawdown`, new trades must be blocked, and exposure must be reduced.

## Consistency Rules
The `RecommendationConsistencyValidator` enforces strict consistency across the recommendations outputted by independent engines.
- **Contradiction**: `BlockOutcome::Freeze` AND `IncreaseOutcome::Increase`. (Cannot increase exposure when trading is frozen).
- **Contradiction**: `is_critical_drawdown == true` AND `IncreaseOutcome::Increase`. (Cannot increase exposure during a critical drawdown).
- **Contradiction**: `is_frozen == true` AND `CloseOutcome != EmergencyLiquidation`. (Frozen portfolio requires EmergencyLiquidation).
- **Contradiction**: `BlockOutcome::Block` AND `IncreaseOutcome::Increase`. (Cannot increase exposure when new trades are blocked).

## Explanation Completeness
- Every outcome must be accompanied by a `RecommendationExplanation`.
- The explanation must clearly state the `most_contributing_factor` without any ambiguity. No black boxes.

## Recovery Mode
- Recovery behavior must restrict sizing/allocation recommendations (`AllowReduced`, `Delay`) until health, quality, and heat fully recover over a sustained period. This maps to the gradual degradation of portfolio heat and the eventual stabilization of the drawdown depth.

## Failsafe Mechanisms
- The engine guarantees it will never panic under any state configuration.
- The engine operates purely on explicitly defined variables, guaranteeing deterministic behavior at all times.
