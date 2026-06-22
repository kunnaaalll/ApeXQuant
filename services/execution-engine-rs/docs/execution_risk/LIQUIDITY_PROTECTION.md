# Liquidity Protection

Analyzes real-time top of book metrics.

## Evaluated Fields
- Book depth
- Spread quality
- Order imbalance
- Available liquidity

## Regimes
- `Excellent`
- `Normal`
- `Weak`
- `Poor`
- `Broken`

If liquidity reaches the `Broken` state, it automatically forces the overarching state to `Critical` or `Frozen`.
