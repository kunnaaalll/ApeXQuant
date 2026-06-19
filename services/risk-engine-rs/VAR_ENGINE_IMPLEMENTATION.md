# VaR and Tail Risk Engine (Phase 4)

## Institutional Rationale
The Value-at-Risk (VaR) and Tail Risk subsystem represents Phase 4 of the APEX V3 Risk Engine. It provides institutional-grade downside measurement. This enables proactive management of catastrophic loss exposure without relying on standard float-based approximations, ensuring 100% precision and determinism.

## VaR Philosophy
We implement both Historical and Parametric VaR.
- **Historical VaR**: Uses actual observed rolling loss windows to determine the percentile loss at various confidence levels (90%, 95%, 99%, 99.9%).
- **Parametric VaR**: Assumes a normal distribution, dynamically tracking mean return and variance using Welford's online algorithm to estimate volatility.

## Expected Shortfall Theory
Expected Shortfall (CVaR) measures the average loss beyond the VaR threshold. It addresses VaR's inability to quantify the severity of tail events. We strictly enforce the mathematical invariant `ExpectedShortfall >= VaR`.

## Tail Risk Modelling
Tail Risk tracks the absolute worst-case losses, summing tail event magnitudes, and evaluating the frequency of extreme downside events. This computes a bounded `tail_risk_score` from `0 -> 100`.

## Severity Thresholds
State changes escalate based on quantifiable metrics:
1. `Normal`
2. `Elevated`
3. `High`
4. `Critical`
5. `Collapse`

## Replay Architecture
Through the `VarRiskSnapshot` and `VarRiskEvent` implementations, every state transition and risk parameter is fully replayable. Snapshots maintain the absolute state in an immutable construct.
