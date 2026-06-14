# RESEARCH ENGINE IMPLEMENTATION

## Purpose

The Research Engine continuously surfaces **where APEX should focus next** — and what it should avoid. It produces ranked lists of opportunities, best edges, and worst performers across all analytical dimensions: symbol, session, regime, timeframe, and pattern.

---

## Components

### `OpportunityRankingEngine` (`opportunity_ranking.rs`)

Ranks candidate opportunities by composite score:

```
score = edge × confidence × sample_quality
```

| Field               | Description                              |
|---------------------|------------------------------------------|
| `dimension`         | "symbol" / "session" / "regime" / etc.  |
| `label`             | Specific value (e.g. "EURUSD", "London")|
| `edge`              | Signed expectancy in R-multiples        |
| `confidence`        | [0, 1] from confidence engine           |
| `sample_quality`    | [0, 1] penalty from SampleBiasDetector  |
| `historical_evidence` | Trade count backing this ranking      |

**Tiebreak:** Lexicographic by `label` — guarantees deterministic stable ordering.

---

### `EdgeRankingEngine` (`edge_ranking.rs`)

Ranks the best-performing strategy × dimension combinations:

```
score = expectancy × profit_factor × confidence
```

| Field           | Description                     |
|-----------------|---------------------------------|
| `strategy_id`   | Strategy UUID as string         |
| `expectancy`    | R-multiple expectancy           |
| `profit_factor` | Gross profit / gross loss       |
| `sharpe_approx` | Risk-adjusted return proxy      |
| `confidence`    | [0, 1]                          |

**Tiebreak:** `strategy_id` then `label` (lexicographic).

---

### `WeaknessRankingEngine` (`weakness_ranking.rs`)

Ranks the **worst** performers — designed to enforce avoidance:

```
score = expectancy × profit_factor
```

Sorted **ascending** (most negative = rank 1 = worst first).

#### `WeaknessState` Classification

| State       | Condition                                                              |
|-------------|------------------------------------------------------------------------|
| `Forbidden` | expectancy < −0.10 **OR** profit_factor < 0.70 **OR** drawdown > 30%  |
| `Danger`    | expectancy < −0.03 **OR** profit_factor < 0.90 **OR** drawdown > 20%  |
| `Weak`      | expectancy < 0.02 **OR** profit_factor < 1.00                         |
| `Watchlist` | Everything else                                                        |

**`Forbidden`** means the dimension must not be traded. It is checked before `Danger`.

---

## Determinism Guarantees

All three ranking engines are **deterministic**:

1. Sort is by `Decimal` comparison — exact, no floating-point ambiguity.
2. Equal scores break on `label` (lexicographic) — always unique resolution.
3. Input order does not affect output ranks.
4. 10,000-pass determinism test verified for `OpportunityRankingEngine`.

---

## Research Workflow

```
1. Collect TradeRecord[] by dimension (symbol, regime, session, etc.)
2. Run ReplayEngine::replay() per dimension → per-dimension ReplayResult
3. Apply SampleBiasDetector + ConfidencePenaltyEngine per dimension
4. Construct OpportunityRanking / EdgeRanking / WeaknessRanking structs
5. Call ranking engine → get ordered Vec<>
6. Surface top-N opportunities and bottom-N weaknesses for research
```

---

## Usage in MetaRecommendationEngine

The Research Engine output feeds directly into `MetaRecommendationInput`:

- `OpportunityRanking[0]` → best symbol/session to allocate to
- `WeaknessRanking` with `Forbidden` → dimension blocked from new trades
- `EdgeRanking[0]` → largest contributor to `MetaRecommendation::largest_contributor`
