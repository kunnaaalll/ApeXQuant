# META INTELLIGENCE TESTS

## Test Coverage Summary

Phase 5 introduces **7 integration test modules** and **2 embedded unit test modules**, covering every new engine in the Meta Intelligence layer.

---

## Test Modules

| Module                        | Location                                              | Tests |
|-------------------------------|-------------------------------------------------------|-------|
| `meta_tests`                  | `tests/meta/meta_tests.rs`                           | 12    |
| `meta_recommendation_tests`   | `tests/meta/meta_recommendation_tests.rs`            | 11    |
| `counterfactual_tests`        | `tests/counterfactual/counterfactual_tests.rs`       | 9     |
| `evolution_tests`             | `tests/evolution/evolution_tests.rs`                 | 11    |
| `degradation_tests`           | `tests/degradation/degradation_tests.rs`             | 9     |
| `overfitting_tests`           | `tests/overfitting/overfitting_tests.rs`             | 9     |
| `research_tests`              | `tests/research/research_tests.rs`                   | 9     |
| `simulator_tests`             | `tests/simulator/simulator_tests.rs`                 | 7     |

**Total: 77 integration tests**

---

## Test Categories by Type

### Unit Tests — Correctness

Verify each function produces the mathematically correct output:

```
health_collapse_is_immediate            → collapse drops score to 0 in one transition
health_recovery_is_gradual              → recovery bounded to +5 per cycle
counterfactual_difference_computed      → difference == alternate - actual exactly
parameter_comparison_best_worst         → best/worst are correctly identified
collapse_triggered_on_negative_exp      → severity ≥ 70 triggers Triggered signal
edge_decay_collapse_on_large_drop       → total decay > 0.60 → Collapse state
```

### Property Tests — Invariants

Verify mathematical invariants hold across all valid inputs:

```
health_clamped_to_100                   → score never exceeds 100
overfit_penalty_never_exceeds_one       → all 100 parameterised cases ≤ 1.0
confidence_clamped_above_zero           → penalty never negative
weakness_forbidden_on_severe            → classification is always applied
opportunity_rank_is_contiguous_from_one → ranks are 1..N exactly
```

### Determinism Tests — Stability

Verify identical inputs always produce identical outputs:

```
health_monte_carlo_determinism          → 10,000 identical calls, zero divergence
recommendation_deterministic_10k        → 10,000 calls, same action + reason
all_evolution_engines_deterministic_10k → 10,000 calls per engine
alternate_history_deterministic         → 1,000 calls, same difference
parameter_comparison_deterministic      → 1,000 calls, same best_variant
replay_monte_carlo_determinism_100k     → 100,000 calls, zero divergence (the largest test)
```

### Stress Tests — Edge Cases

Verify correct behaviour at boundaries:

```
replay_empty_set_returns_empty          → trade_count=0, expectancy=0
comparison_empty_returns_none           → no profiles → None
parameter_comparison_requires_two       → single variant → None
regime_needs_min_two_windows            → single window → None
edge_decay_needs_at_least_two           → single snapshot → None
strategy_degradation_needs_two          → single window → None
config_evaluator_empty_returns_none     → empty variant list → None
```

---

## Monte Carlo Validation

The `replay_monte_carlo_determinism_100k` test is the primary stress test:

- **Inputs:** 100 trades (deterministic pattern: win if `i % 3 != 0`)
- **Filter:** no filter (all trades)
- **Iterations:** 100,000
- **Assertions:** `expectancy`, `win_rate`, `profit_factor` identical to reference across all runs
- **Purpose:** Confirms that `ReplayEngine` has zero divergence even under sustained load

---

## Running Tests

```bash
# All Phase 5 tests
cargo test 2>&1

# Only Monte Carlo test
cargo test monte_carlo -- --nocapture

# Only degradation tests
cargo test degradation_tests

# Only simulator tests  
cargo test simulator_tests
```

---

## Expected Results

```
test result: ok. 77 passed; 0 failed; 0 ignored
```

Zero panics. Zero unsafe. Zero randomness. 100% deterministic.
