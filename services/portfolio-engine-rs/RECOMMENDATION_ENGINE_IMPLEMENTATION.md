# Recommendation Engine Implementation

## Philosophy
The Recommendation Engine acts as the hedge fund investment committee for the APEX V3 Portfolio Engine. Its primary goal is to **protect long-term capital**, prioritizing longevity over aggressive growth. It acts upon the data provided by the Heat, Health, Quality, and Drawdown engines to issue portfolio-level mandates.

## Engine Structure
The Recommendations Subsystem generates four primary directives:

1. **Increase Exposure Recommendation**: Decides if the portfolio should take on more risk (`Increase`, `Maintain`, `Delay`, `Reject`).
2. **Reduce Exposure Recommendation**: Instructs when to scale back due to stress or heat (`NoAction`, `ReduceSlightly`, `ReduceModerately`, `ReduceAggressively`, `EmergencyReduction`).
3. **Close Exposure Recommendation**: Determines when specific positions or correlated clusters must be cut entirely to protect the portfolio (`Hold`, `CloseWeakPositions`, `CloseCorrelatedPositions`, `CloseHighRiskPositions`, `EmergencyLiquidation`).
4. **Block Engine**: Governs whether new capital can be deployed into fresh trades (`Allow`, `AllowReduced`, `Delay`, `Block`, `Freeze`).

## Explainability
Every recommendation is completely transparent. There are no black-box outputs. Each assessment comes with a `RecommendationExplanation` struct containing:
- **Why**: A plain-text explanation of the rationale.
- **What Changed**: The trigger or condition that drove the decision.
- **Contributing Factor**: The primary portfolio metric (e.g., "Heat", "Health", "Drawdown") driving the outcome.
- **What Prevented A Stronger Recommendation**: Explicit reasoning why a more aggressive or lenient stance was not taken.

## Determinism and Speed
The recommendation engine evaluates logic purely mathematically based on input scores (0-100) and booleans (`is_frozen`, `is_critical_drawdown`). It has zero external dependencies, making it 100% deterministic, panic-free, and capable of operating with sub-millisecond latency.
