# Memory System Implementation

## Overview
The `memory` module maintains the historical evidence required for exponential smoothing without accumulating unbounded raw event logs in hot memory. It implements an explicit, replayable context buffer.

## Architecture
- `edge_memory.rs`: Employs a bounded ring-buffer (`VecDeque`) coupled with an online exponential smoothed aggregator. Old information decays implicitly via the smoothing factor (`alpha`) without storing infinite history.
- `context_memory.rs`: Tracks smoothed scores associated with particular contexts (e.g., market regimes, specific timeframes) using a deterministic map.
- `performance_memory.rs`: Retains a strict window of the most recent `PerformanceSnapshot` objects to enable tracking momentum and detecting rapid expectancy deterioration over short, discrete periods.

## Replayability Guarantees
Because `MemorySystem` explicitly caps capacity and relies solely on `rust_decimal` arithmetic, replaying a known sequence of trades (the event log) from a zero-state will deterministically reproduce the exact memory state at any given point in time.
