# MICROSTRUCTURE SCORE MODEL

## Overview
Produces a definitive 0-100 `MicrostructureScore` evaluating the overall favorability of the market for execution.

## Calculation
Aggregates six 0-100 sub-scores evenly:
1. Spread Score
2. Imbalance Score
3. Depth Score
4. Resiliency Score
5. Volatility Score
6. Cost Score

## Grade Brackets
- `Elite`: >= 90
- `Strong`: >= 75
- `Normal`: >= 50
- `Weak`: >= 25
- `Poor`: < 25

This score is purely an assessment matrix; no execution choices are natively made at this level.
