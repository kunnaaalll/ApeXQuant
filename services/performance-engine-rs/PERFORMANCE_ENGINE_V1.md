# APEX PERFORMANCE ENGINE V1

## Objective
The APEX Performance Engine V1 acts as the ultimate truth-measurement layer for the APEX V3 architecture. Designed for rigorous institutional-grade analysis, it does **not** trade, execute, or learn. Instead, it measures strategy edge, regime effectiveness, capital efficiency, and system degradation. It operates with 100% determinism, evaluating complete historical trades and portfolio evolution.

## Core Mandates
1. **No AI/Black Boxes:** Every calculation is deterministic and purely mathematical.
2. **Zero Randomness:** Given the same historical sequence of events, the outcome is always identical.
3. **Truth First:** Designed to emulate institutional hedge fund quantitative measurement capabilities (e.g., Renaissance Technologies, Citadel).

## Architecture & Modules
### Expected Value & Edge
- **Expectancy Engine:** Measures rolling, daily, weekly, and trade expectancy (States: Exceptional, Strong, Normal, Weak, Negative).
- **Edge Engine:** Determines true edge, acceleration, and degradation. Identifies true signal vs. noise.

### Contextual Analysis
- **Regime Engine:** Explains performance across market states (Trending, Ranging, Expansion, Contraction, Volatility).
- **Session Engine:** Measures profitability during Asia, London, NY, and overlap sessions.
- **Symbol & Timeframe Engines:** Tracks edge, stability, and expectancy across assets and multi-timeframe domains.
- **Setup & SL/TP Engines:** Explains which setups (e.g., BOS, FVG, Liquidity Sweeps) are profitable, what RR combinations work best, and where capital is being inefficiently managed.
- **Confidence Engine:** Checks human/system pre-trade confidence calibration against true win rates.

### Health & Stability
- **Degradation Engine:** Detects shifts and edge decay (Healthy, Watch, Warning, Critical).
- **Stability Engine:** Measures variance and risk-adjusted returns (Sharpe, Sortino, Calmar, Ulcer Index).
- **Streaks, Drawdown, Psychology:** Monitors capital leaking during drawdown phases and sequential variance.

## Storage
- **Append-only Postgres Store:** Immutable logs.
- **Snapshot & Event Sourcing:** For O(1) state reconstruction and high-performance replays.

## Performance Constraints
- Average Latency: `<5ms`
- P99 Latency: `<20ms`
- Panics: `0`
- Unsafe Code: `0`
