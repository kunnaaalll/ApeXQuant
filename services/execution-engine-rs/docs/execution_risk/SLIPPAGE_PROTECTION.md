# Slippage Protection

Monitors the realized vs. expected slippage of orders.

## Metrics
- Expected slippage.
- Realized slippage.
- Slippage drift.

## Regimes
- **Healthy**
- **Elevated**
- **Danger**
- **Collapse**

Calculates a bounded penalty score `0-100`. Values are clamped, and negative slippage is not permitted.
