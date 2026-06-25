# Structure Engine

The Structure engine attempts to abstract market micro-structures into macro-states (Consolidation, Expansion, Breakdown, Breakout).

## Heuristics

- Tracks moving highest-high and lowest-low values over the specified period.
- Tests current prices against these channel boundaries.
- Uses `StructureState` to summarize channel behavior. Breakouts and Breakdowns require distinct price breaches of the recent rolling ranges.
