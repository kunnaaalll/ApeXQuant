# META INTELLIGENCE INVARIANTS

## Overview

The following invariants are **mathematical guarantees** that must hold at all times throughout Phase 5. A violation of any invariant constitutes a **system defect** and must block deployment.

---

## I. Health Score Invariants

| ID   | Invariant                                                              | Module                  |
|------|------------------------------------------------------------------------|-------------------------|
| H-01 | `health.score ∈ [0, 100]` — always clamped                           | `strategy_health.rs`    |
| H-02 | Collapse is immediate: if `raw < prev`, `next = raw` with no delay    | `strategy_health.rs`    |
| H-03 | Recovery is gradual: `next ≤ prev + 5` per cycle                     | `strategy_health.rs`    |
| H-04 | If `expectancy < -0.10 OR pf < 0.70`, `synthesise()` returns `0`    | `strategy_health.rs`    |
| H-05 | `synthesise()` is pure: same inputs → same output, always             | `strategy_health.rs`    |

---

## II. Evolution Invariants

| ID   | Invariant                                                              | Module                  |
|------|------------------------------------------------------------------------|-------------------------|
| E-01 | `evaluate()` returns `None` if `windows.len() < 2`                   | All evolution modules   |
| E-02 | `Collapsing` state takes priority over all others                     | `strategy_evolution.rs` |
| E-03 | `Abandoned/Obsolete/Exhausted` triggered when `last.trade_count == 0`| All domain engines      |
| E-04 | `expectancy_drift = last.expectancy - first.expectancy` exactly       | All domain engines      |
| E-05 | Output is fully determined by ordered window slice — no hidden state   | All evolution modules   |

---

## III. Degradation Invariants

| ID   | Invariant                                                              | Module                     |
|------|------------------------------------------------------------------------|----------------------------|
| D-01 | `DegradationState::Collapse` triggers on `last.expectancy < -0.05 OR last.pf < 0.80` | `strategy_degradation.rs` |
| D-02 | `CollapseSignal::Triggered` fires when `severity_score >= 70`         | `collapse_detector.rs`     |
| D-03 | `severity_score ∈ [0, 100]` — clamped before return                   | `collapse_detector.rs`     |
| D-04 | `EdgeDecayState::Collapse` fires when `last.edge_score < 0.20 OR total_decay > 0.60` | `edge_decay.rs` |
| D-05 | All engines require `≥ 2` inputs — return `None` otherwise            | All degradation modules     |

---

## IV. Overfitting Invariants

| ID   | Invariant                                                              | Module                      |
|------|------------------------------------------------------------------------|-----------------------------|
| O-01 | `confidence_penalty ∈ [0, 1]` — never negative, never > 1             | `overfit_detector.rs`       |
| O-02 | All penalty factors are multiplicative — no additive composition       | `confidence_penalty.rs`     |
| O-03 | `oos_trades < 30` always applies a penalty — no bypass                 | `overfit_detector.rs`       |
| O-04 | `sample_penalty` is monotonically non-decreasing with `trade_count`    | `sample_bias.rs`            |
| O-05 | `apply(raw, penalty)` always returns `raw × combined_penalty ≤ raw`   | `confidence_penalty.rs`     |

---

## V. Research Ranking Invariants

| ID   | Invariant                                                              | Module                       |
|------|------------------------------------------------------------------------|------------------------------|
| R-01 | Ranks are contiguous starting at 1: `{1, 2, 3, …, N}` exactly        | All ranking engines           |
| R-02 | Tiebreak is always lexicographic by label — no randomness              | All ranking engines           |
| R-03 | `WeaknessState::Forbidden` is checked before `Danger`, before `Weak`  | `weakness_ranking.rs`        |
| R-04 | `OpportunityRanking` composite score = `edge × confidence × sample_quality` | `opportunity_ranking.rs` |
| R-05 | Equal inputs produce equal rank orderings across all calls             | All ranking engines           |

---

## VI. Counterfactual Invariants

| ID   | Invariant                                                              | Module                        |
|------|------------------------------------------------------------------------|-------------------------------|
| C-01 | `difference == alternate_outcome - actual_outcome` — no rounding       | `what_if.rs`                  |
| C-02 | `confidence` is caller-supplied — engine never modifies it             | All counterfactual modules    |
| C-03 | No forward projection — all inputs must be historical trade data       | All counterfactual modules    |
| C-04 | `ParameterComparisonEngine` requires `≥ 2` variants — `None` otherwise | `parameter_comparison.rs`    |

---

## VII. Replay Simulator Invariants

| ID   | Invariant                                                              | Module                        |
|------|------------------------------------------------------------------------|-------------------------------|
| S-01 | Replay output is fully determined by `(trades, filter)` — no state    | `replay_engine.rs`            |
| S-02 | `trade_count == 0` returns `ReplayResult::empty()` — no panic          | `replay_engine.rs`            |
| S-03 | `profit_factor` is capped at `999` when `gross_loss == 0`             | `replay_engine.rs`            |
| S-04 | `max_drawdown ∈ [0, 1]` — computed from equity curve, never negative  | `replay_engine.rs`            |
| S-05 | 100,000-run Monte Carlo shows zero divergence in all metrics           | `simulator_tests.rs`          |

---

## VIII. Meta Recommendation Invariants

| ID   | Invariant                                                              | Module                       |
|------|------------------------------------------------------------------------|------------------------------|
| M-01 | `Retire` takes priority over all other actions                         | `meta_recommendation.rs`     |
| M-02 | `confidence ∈ [0, 1]` — never exceeds 1.0 in any action path          | `meta_recommendation.rs`     |
| M-03 | Every recommendation includes a non-empty `reason` string             | `meta_recommendation.rs`     |
| M-04 | Decision tree is evaluated top-to-bottom — first matching gate wins   | `meta_recommendation.rs`     |
| M-05 | Identical inputs always produce identical `action + reason`            | `meta_recommendation.rs`     |

---

## Certification Requirement

All invariants above are verified by the test suite in `tests/`. Before Phase 6 can begin:

- All 77 tests must pass with `cargo test`
- `cargo check` must return zero errors and zero warnings
- The Monte Carlo test (`replay_monte_carlo_determinism_100k`) must complete without panic
