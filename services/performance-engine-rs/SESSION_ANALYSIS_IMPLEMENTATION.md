# SESSION ANALYSIS IMPLEMENTATION

## Session Mathematics
Analyzes trading performance bounded by explicit temporal overlaps (Asia, London, New York, London-New York overlap).

### Ranking Methodology
Sessions are ranked primarily on normalized expectancy and secondarily on profit factor.

### Overfitting Prevention
- A strict win-rate requirement (>= 50%) is added for a session to achieve `Exceptional` status. A high profit factor alone on a few lucky trades will not qualify.
- `MIN_TRADES_FOR_EVALUATION = 30` prevents single-day outliers from distorting session statistics.

### Edge Cases
- Trades spanning multiple sessions are classified according to entry execution time.
- Zero-trade sessions output an expectancy of `Decimal::ZERO` and state `Normal`.
