# PORTFOLIO ANALYTICS INVARIANTS

The core philosophy of the APEX Analytics engine is absolute safety. Institutional metrics must never crash the system or corrupt the dataset.

## Core Invariants

- **Profit Factor**: Guaranteed to be `>= 0`.
- **Ratios (Sharpe/Sortino/Calmar)**: Must be finite `f64`. Cannot be infinite.
- **Division by Zero**: Handled explicitly in the `math::safe_divide` function. Fallback defaults are explicitly defined (usually `0.0`).
- **NaN Prevention**: Any calculation resolving to `NaN` is scrubbed and reverts to `0.0`.
- **Probabilities (Win Rate/Loss Rate)**: Strictly clamped between `0.0` and `1.0`.
- **State Transition**: All state transitions require an explicit `AnalyticsEvent`. No silent mutations exist.

## Failure Scenarios & Edge Cases

1. **First Trade Scenario**: Division by zero for win rate and average win/loss is protected; engine will report `0.0` metrics until valid counts exist.
2. **Never Losing**: Profit factor would divide by zero. Protected by `safe_divide`.
3. **Zero Volatility**: Sharpe and Sortino denominators are zero. Protected by `safe_divide`.
4. **Floating Point Imprecision**: Direct equality checks (`==`) are discouraged for logic control; all analytical branching uses relative comparisons or bounding values.
