# APEX V3 Portfolio Heat Engine Tests

The testing philosophy for the Portfolio Heat Engine demands total determinism and robust boundary enforcement.

## Unit Testing

Unit tests directly exercise the public API of the `PortfolioHeat` and `HeatDecayModel` structs:

1. **Bounding Tests**: Asserts that `PortfolioHeat` instances generated with arbitrary theoretical scores are tightly clamped between `0` and `100`.
2. **Decay Mechanics**: Validates that cooldown durations are respected, and heat decreases exactly as specified once the threshold ticks are met.
3. **Risk Budgeting**: Asserts that `RiskBudget::can_allocate` responds correctly to capacity constraints, ensuring reserved and emergency risk pools are strictly off-limits.

## Property & Stress Testing

We employ the `proptest` framework to generate thousands of combinations of factor weights, raw scores, and account parameters:
- **Fuzzing Score Components**: Pushing extreme values (e.g. 100% correlation) to verify `PortfolioHeat` remains within bounds.
- **Margin Spikes**: Generating unrealistic margin requirements to ensure the `CapitalPressureAssessment` gracefully limits risk without panicking.

## Replay and Determinism

As the Heat Engine calculates its output exclusively via the sum of deterministic factor scores derived from the `HeatEvent` stream, a specific sequence of immutable `HeatSnapshot` events guarantees 100% replayability.

- **Replay Tests** will ingest CSV dumps of historical trading days and verify that the exact sequence of `HeatEvent` triggers produces the exact expected `PortfolioHeat` timeline down to the final decimal of contribution.
