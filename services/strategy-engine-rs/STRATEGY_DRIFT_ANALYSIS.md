# Drift Analysis

Drift logic safely compares running strategies against baseline or reference strategies to quantify behavioral drift over time.

## Absolute Drift

Simple scalar difference: `abs(current - reference)`

## Relative Drift

Formula: `difference / max(abs(reference), 1) * 100`

- Prevents division by zero.
- Ensures stability around small values.
- Strictly clamped between 0 and 100.
