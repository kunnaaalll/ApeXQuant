# STRATEGY HEALTH IMPLEMENTATION

## Purpose

`StrategyHealth` provides a single, human-readable **0–100 score** that summarises the total condition of a strategy. It is the primary input to the `MetaRecommendationEngine` and controls allocation decisions.

---

## Health Score Components

```
score = win_rate_pts + expectancy_pts + profit_factor_pts + drawdown_pts + confidence_pts + stability_pts
```

| Component           | Max Points | Formula                                                        |
|---------------------|------------|----------------------------------------------------------------|
| Win Rate            | 20         | `win_rate × 20`, capped at 20                                  |
| Expectancy          | 25         | `expectancy / 0.10 × 25` if `expectancy ∈ [0, 0.10]`, else 25 |
| Profit Factor       | 20         | `(pf - 1) / 1 × 20` if `pf ∈ [1, 2]`, else 20                |
| Drawdown (penalty)  | 15         | Linear decay from 15 (dd=5%) to 0 (dd≥20%)                    |
| Confidence          | 10         | `confidence × 10`, capped at 10                                |
| Stability           | 10         | `stability × 10`, capped at 10                                 |

**Total maximum: 100 points.**

---

## State Classification

| Score     | State     |
|-----------|-----------|
| 90 – 100  | Excellent |
| 70 – 89   | Healthy   |
| 50 – 69   | Normal    |
| 30 – 49   | Weak      |
| 10 – 29   | Critical  |
| 0 – 9     | Dead      |

---

## Collapse vs. Recovery Model

### Collapse — Immediate

If `expectancy < -0.10 OR profit_factor < 0.70`, the synthesised score is **immediately set to 0**.

In the transition model, collapse is always fully applied:
```rust
if clamped < prev {
    next_score = clamped;  // immediate — no rate limiting
}
```

### Recovery — Gradual

Recovery is rate-limited to **5 points per evaluation cycle** (`MAX_RECOVERY_STEP = 5`):
```rust
let increase = clamped.saturating_sub(prev);
next_score = prev.saturating_add(increase.min(MAX_RECOVERY_STEP));
```

This means a strategy collapsed from 90 → 0 requires **18 consecutive clean cycles** to fully recover. This is intentional — it prevents "health washing" from a single good period.

---

## Drawdown Penalty Formula

```
if drawdown ≤ 5%:   pts = 15
if drawdown ≥ 20%:  pts = 0
else:               pts = (1 - (drawdown - 0.05) / 0.15) × 15
```

This creates a linear interpolation between 5% (max health) and 20% (zero contribution).

---

## Invariants

1. `score ∈ [0, 100]` — clamped before return.
2. Collapse triggers on `expectancy < -0.10` OR `pf < 0.70` — immediate, no grace period.
3. Recovery is bounded: `delta ≤ +5` per cycle.
4. `synthesise()` is **pure** — same inputs always produce same `u8` output.
5. Verified: 10,000-pass Monte Carlo determinism test (zero divergence).
6. Score is **u8** — no overflow possible.

---

## Integration

```rust
// 1. Synthesise score from analytics
let raw_score = StrategyHealth::synthesise(
    win_rate, expectancy, profit_factor, max_drawdown, confidence, stability
);

// 2. Apply gradual recovery / immediate collapse
let next_health = current_health.transition(raw_score);

// 3. Feed into MetaRecommendationEngine
let input = MetaRecommendationInput {
    health_score: next_health.score,
    ..
};
let recommendation = MetaRecommendationEngine::recommend(&input);
```
