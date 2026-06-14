# SYMBOL ANALYSIS IMPLEMENTATION

## Symbol Granularity
Tracks performance of unique ticker symbols to isolate specific edges vs market noise.

### Small Sample Penalty
A symbol requires a minimum of 30 trades. Below this, its state is hard-coded to `Normal` regardless of profitability.

### Edge Scoring Requirement
To achieve `Exceptional` or `Strong` state, a symbol requires:
- Profit Factor > 1.5 (Strong) or > 2.0 (Exceptional)
- Positive expectancy
- Edge Score > 0.5 (Strong) or > 0.8 (Exceptional)

This guarantees that we do not mistakenly allocate capital to a symbol that profited massively off a single lucky outlier trade.
