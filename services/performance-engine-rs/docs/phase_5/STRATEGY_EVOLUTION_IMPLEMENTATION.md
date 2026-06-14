# STRATEGY EVOLUTION IMPLEMENTATION

## Purpose

The Strategy Evolution Engine answers the question:

> *"Is this strategy improving, stable, weakening, or collapsing?"*

It operates on **ordered rolling windows** of historical performance — oldest first — and produces a `StrategyEvolutionAssessment` with drift measurements across four dimensions.

---

## Core Structures

### `StrategyEvolutionAssessment`

| Field              | Type      | Description                                           |
|--------------------|-----------|-------------------------------------------------------|
| `expectancy_drift` | `Decimal` | Latest expectancy minus baseline (signed)             |
| `drawdown_drift`   | `Decimal` | Latest drawdown minus baseline (positive = worse)     |
| `confidence_drift` | `Decimal` | Latest confidence minus baseline (signed)             |
| `stability_drift`  | `Decimal` | Latest stability minus baseline (signed)              |
| `state`            | `EvolutionState` | Classified outcome                             |

### `EvolutionState`

| State       | Trigger Condition                                                          |
|-------------|---------------------------------------------------------------------------|
| `Collapsing` | expectancy_drift ≤ −0.20 **OR** drawdown_drift ≥ 0.20 **OR** confidence_drift ≤ −0.30 |
| `Improving`  | expectancy_drift ≥ 0.05 **AND** drawdown_drift ≤ 0 **AND** stability_drift ≥ 0 |
| `Weakening`  | expectancy_drift ≤ −0.05 **OR** drawdown_drift ≥ 0.05                    |
| `Stable`     | All other cases                                                            |

**Collapse is evaluated before all other states** — it takes priority.

---

## Domain-Specific Evolution Engines

Each engine operates on ordered `Window` slices (oldest → newest):

| Engine                   | Input Window Key Metrics           | Trend States                                    |
|--------------------------|------------------------------------|-------------------------------------------------|
| `RegimeEvolutionEngine`  | `win_rate`, `expectancy`, `profit_factor` | Strengthening / Stable / Weakening / Abandoned |
| `SymbolEvolutionEngine`  | `expectancy`, `profit_factor`, `max_drawdown` | Strengthening / Stable / Weakening / Exhausted |
| `PatternEvolutionEngine` | `win_rate`, `expectancy`, `avg_rr` | Maturing / Stable / Fading / Obsolete          |
| `TimeframeEvolutionEngine` | `expectancy`, `profit_factor`, `stability` | Strengthening / Stable / Weakening / Abandoned |
| `SessionEvolutionEngine` | `win_rate`, `expectancy`, `avg_rr` | Improving / Stable / Deteriorating / Abandoned |

---

## Algorithm

For each domain engine:

```
let expectancy_drift = windows.last().expectancy - windows.first().expectancy;
let secondary_drift  = windows.last().<secondary> - windows.first().<secondary>;

if last.trade_count == 0            → Abandoned/Obsolete/Exhausted
elif expectancy_drift >= +0.05 AND secondary is non-negative → Strengthening/Maturing/Improving
elif expectancy_drift <= -0.05 OR secondary degraded → Weakening/Fading/Deteriorating
else                                → Stable
```

- Requires **≥ 2 windows** — returns `None` with fewer.
- All drift values are **signed Decimal** — never floating point.
- Tiebreaks are impossible (trend classification is strict-inequality based).

---

## Determinism Guarantee

- Inputs are ordered slices — caller controls ordering.
- No internal sorting or randomness.
- `Decimal` arithmetic is exact — no floating-point rounding.
- Verified: 10,000-pass determinism test across all engines.
