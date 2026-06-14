# COUNTERFACTUAL ENGINE IMPLEMENTATION

## Purpose

The Counterfactual Engine answers questions of the form:

> *"What would have happened if we had traded only the London session?"*
> *"What would have happened with a 3:1 RR instead of 2:1?"*
> *"What would the outcome have been in a trending regime?"*

**No prediction. Historical replay only.**

---

## Core Components

### `CounterfactualResult` (`what_if.rs`)

The base output type shared by all counterfactual evaluations.

| Field              | Type      | Description                                    |
|--------------------|-----------|------------------------------------------------|
| `actual_outcome`   | `Decimal` | The measured historical result                 |
| `alternate_outcome`| `Decimal` | The hypothetical result under alternate context |
| `difference`       | `Decimal` | `alternate_outcome - actual_outcome` (signed)  |
| `confidence`       | `Decimal` | [0, 1] — how much historical evidence backs this |
| `reason`           | `String`  | Human-readable explanation                     |

**Note:** `difference > 0` means the alternate would have been *better*.

---

### `AlternateHistoryEngine` (`alternate_history.rs`)

Evaluates a specific historical context alternative.

**Input:** `AlternateHistoryContext` — specifies which dimension differs:
- `session_id`, `regime_id`, `symbol_id`, `timeframe`, `pattern_id` — all `Option<String>`

**Algorithm:**
```
CounterfactualResult::new(
    actual_expectancy,
    alternate_expectancy,
    confidence,
    reason: format!("Alternate context: {:?}", context)
)
```

The caller computes both `actual_expectancy` and `alternate_expectancy` from `ReplayEngine::replay()` with appropriate filters — the engine assembles the result and computes the signed difference.

---

### `ParameterComparisonEngine` (`parameter_comparison.rs`)

Compares alternative SL/TP/RR/filter configurations against the same historical trade set.

**`ParameterVariant` fields:**
| Field           | Description                              |
|-----------------|------------------------------------------|
| `variant_id`    | Unique identifier                        |
| `sl`            | Stop loss in pips/ticks                  |
| `tp`            | Take profit in pips/ticks                |
| `rr`            | Risk:reward ratio                        |
| `filter_score`  | Entry filter quality threshold           |
| `entry_quality` | Minimum entry quality threshold          |
| `outcome`       | Historical expectancy for this variant   |

**`ParameterComparisonResult` fields:**
| Field          | Description                    |
|----------------|--------------------------------|
| `best_variant` | Highest-outcome configuration  |
| `worst_variant`| Lowest-outcome configuration   |
| `difference`   | `best.outcome - worst.outcome` |
| `confidence`   | Caller-supplied [0, 1]         |

- Requires **≥ 2 variants** — returns `None` otherwise.
- Sorted by `outcome` descending — fully deterministic.

---

## Integration with ReplayEngine

The canonical workflow for counterfactual analysis:

```rust
// 1. Replay actual configuration
let actual = ReplayEngine::replay(&all_trades, &actual_filter);

// 2. Replay alternate configuration
let alternate = ReplayEngine::replay(&all_trades, &alternate_filter);

// 3. Build counterfactual result
let result = CounterfactualResult::new(
    actual.expectancy,
    alternate.expectancy,
    min(actual_confidence, alternate_confidence),
    "Alternate: London session only vs. all sessions".to_string(),
);
```

---

## Invariants

1. `difference == alternate_outcome - actual_outcome` — exact, no rounding.
2. `confidence` is caller-supplied — the engine never inflates it.
3. All results are **historical only** — no forward projection.
4. Requires `trade_count > 0` in both replays for meaningful confidence.
