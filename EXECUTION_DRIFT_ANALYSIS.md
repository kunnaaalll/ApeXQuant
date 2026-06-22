# Execution Drift Analysis

Drift is tracked across two primary axes:

## Absolute Drift
Measured as `|Expected - Actual|`. This bounds the pure numerical error the Execution logic experienced against the real trade output.

## Relative Drift
Measured as a percentage over the `Expected` basis.
To prevent panic-crashes, Division-By-Zero scenarios (where expected basis was `0`) fallback to a strictly clamped `100%` maximum drift if `Actual > 0`.

## Drift Bounds Mapping
- `None` (0%)
- `Low` (<= 1%)
- `Moderate` (<= 5%)
- `High` (<= 10%)
- `Extreme` (> 10%)
