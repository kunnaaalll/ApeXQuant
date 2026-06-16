# Exposure Risk Engine Implementation

The Exposure Risk Engine (Phase 2) implements institutional-grade measurement and control for portfolio exposures, concentration risks, and structural positioning.

## Core States
Risk States transition deterministically based on concentration scores:
- `Normal`: Safe bounds, optimal diversification.
- `Elevated`: Slight imbalance, requires monitoring.
- `High`: Significant concentration, scaling restricted.
- `Critical`: Severe concentration, position sizing strongly penalized.
- `Frozen`: "Collapse" level reached; no new risk can be added until imbalances are resolved.

Once the `Frozen` state is reached, the system will not transition out of it automatically without an external resolution, preventing looping states.

## Exposures Tracked
- **Symbol Exposure**: Gross and net position sizes relative to total capital.
- **Currency Exposure**: Evaluates synthetic decompositions (e.g., EURUSD = Long EUR, Short USD) to prevent hidden USD concentration.
- **Sector Exposure**: Classifies positions into Forex, Indices, Crypto, Metals, Commodities, Equities to identify correlated clusters.
- **Theme Exposure**: Evaluates qualitative and systemic themes (e.g., Risk-On, Inflation, Tech) to catch non-obvious correlations across different sectors.

## Formulas
Concentration Score `[0, 100]` is evaluated as a heuristic of normalized metrics:
`raw = (largest_position_pct * 2) + largest_sector_pct + largest_theme_pct + largest_currency_pct`
`score = (raw / 5).clamp(0, 100)`

Diversification Score `[0, 100]`:
`diversification = 100 - concentration_score`

## Event-Driven Architecture
All transitions are driven by strictly immutable event structs:
- `PositionOpened`
- `PositionClosed`
- `ExposureUpdated`
- `ConcentrationChanged`
- `ClusterDetected`
- `RiskStateChanged`
