# Execution Escalation Model

The escalation engine aggregates the risk profile.

## Methodology
Takes sub-scores from latency, liquidity, spread, failures, rejections, fill quality, and slippage.
Computes a global `execution_risk_score` ranging from `0` to `100`.

## Mapping
- `0-20`: Normal
- `20-40`: Warning
- `40-60`: Restricted
- `60-80`: Critical
- `80-100`: Frozen

All value arithmetic strictly saturates/clamps to bounds to avoid panics.
