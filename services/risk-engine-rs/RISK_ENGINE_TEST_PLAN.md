# Risk Engine Test Plan

## Unit Testing
- [x] VaR state progressions and threshold updates
- [x] CVaR extreme tail state limits
- [x] Kelly fractions boundary enforcement (max 0.5)
- [x] Drawdown metrics and progressive limitations
- [x] Tail Risk catastrophic combined metrics
- [x] Stress Engine fixed scenario impacts
- [x] Scenario Engine deviation logic

## Validation Strategy
- Determinism verification (no `rand`, no non-determinism).
- Fuzz testing against `rust_decimal` overflow conditions.
- Event sourcing snapshot reconciliation tests.
