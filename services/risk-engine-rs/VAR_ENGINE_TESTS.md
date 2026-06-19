# VaR and Tail Risk Engine Testing

## Overview
Our test suite guarantees mathematical precision, zero panics, and specific institutional properties. 

## Deterministic Properties
1. **VaR ≥ 0**: Tests assert VaR can never report a negative loss.
2. **Expected Shortfall ≥ VaR**: ES cannot be less severe than VaR.
3. **Tail Risk Clamping**: Score never exceeds 100 or drops below 0.
4. **Largest Loss >= Average**: The largest loss recorded must always equal or exceed the average tail loss.

## Large Scale Iterations
`test_deterministic_iterations` continuously applies `100,000` data points. It verifies:
- Online algorithms (like Welford's for variance) do not suffer overflow or instability.
- Tail risk accurately captures extremes without state degradation.

## Zero Panic Guarantees
Tests evaluate empty states (`count == 0`), negative counts, and zero standard deviations to ensure operations fail gracefully or return `Decimal::ZERO` rather than `panic!()` or floating point anomalies (like `NaN` or `Infinity`).
