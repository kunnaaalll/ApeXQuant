# META INTELLIGENCE IMPLEMENTATION

## Overview

Phase 5 of the APEX V3 Performance Engine introduces **Meta Intelligence** — a self-improving institutional research platform that continuously interrogates its own performance history to answer:

- Is our edge migrating?
- Which strategies are improving vs. dying?
- What configurations would have performed better?
- Are we becoming overfit?
- How should APEX evolve?

**Everything is deterministic, evidence-based, and zero-randomness.**

---

## Module Architecture

```
src/
├── meta/
│   ├── strategy_registry.rs      # StrategyProfile — complete historical identity
│   ├── strategy_state.rs         # State machine: Elite → Retired
│   ├── strategy_evolution.rs     # Evolution assessment over rolling windows
│   ├── strategy_comparison.rs    # Multi-strategy ranking with full explanation
│   ├── strategy_health.rs        # 0–100 health score with decay/recovery model
│   └── meta_recommendation.rs   # Evidence-based action recommendation engine
├── counterfactual/
│   ├── what_if.rs                # CounterfactualResult — actual vs. alternate
│   ├── alternate_history.rs      # Context-aware alternate history evaluation
│   └── parameter_comparison.rs  # SL/TP/RR/filter variant comparison
├── evolution/
│   ├── regime_evolution.rs       # Regime performance tracking over time
│   ├── symbol_evolution.rs       # Symbol performance tracking over time
│   ├── pattern_evolution.rs      # Pattern performance tracking over time
│   ├── timeframe_evolution.rs    # Timeframe performance tracking over time
│   └── session_evolution.rs     # Session performance tracking over time
├── degradation/
│   ├── strategy_degradation.rs  # Multi-period expectancy/PF decline
│   ├── edge_decay.rs            # Normalised edge score decay tracking
│   └── collapse_detector.rs     # Immediate collapse signal (severity 0–100)
├── overfitting/
│   ├── overfit_detector.rs      # OOS/IS ratio + parameter density analysis
│   ├── sample_bias.rs           # Sample size confidence multipliers
│   └── confidence_penalty.rs   # Multiplicative penalty composition
├── research/
│   ├── opportunity_ranking.rs   # Best-to-worst dimension ranking
│   ├── edge_ranking.rs          # Strategy × dimension edge leaderboard
│   └── weakness_ranking.rs      # Worst performers with Watchlist→Forbidden states
└── simulator/
    ├── replay_engine.rs          # Deterministic historical trade replay
    ├── variant_runner.rs         # Multi-variant parallel evaluation
    └── configuration_evaluator.rs # Best/worst variant selection with explanation
```

---

## Core Principles

| Principle         | Implementation                                         |
|-------------------|-------------------------------------------------------|
| Deterministic     | All scores derived from `rust_decimal::Decimal` arithmetic only |
| Zero randomness   | No `rand`, no sampling, no neural networks            |
| Explainable       | Every output carries a `String` explanation field     |
| Replayable        | All inputs are pure data — no hidden state            |
| Zero panic        | All division guarded; `Option<T>` used where N < minimum |
| Zero unsafe       | `#![forbid(unsafe_code)]` enforced at workspace level |
| Audit trail       | Every recommendation includes `historical_evidence: u32` |

---

## Data Flow

```
TradeRecord[] ──► ReplayEngine ──► ReplayResult
                         │
                         ├──► VariantRunner ──► VariantResult[]
                         │         │
                         │         └──► ConfigurationEvaluator ──► ConfigurationEvaluation
                         │
StrategyProfile ──► StrategyEvolutionAssessment
                 ──► StrategyComparisonEngine ──► ComparisonResult
                 ──► StrategyHealth::synthesise() ──► u8 (0–100)
                         │
                         └──► MetaRecommendationEngine ──► MetaRecommendation
                                        (action + reason + contributor + weakness)
```

---

## MetaRecommendation Decision Tree

```
expectancy < -0.15 OR pf < 0.65 OR health == 0
    ──► Retire (terminal)

health < 20 OR drawdown > 20% OR overfit_penalty < 0.50
    ──► Pause

expectancy_drift > 8% OR health < 40 OR pf < 1.20
    ──► Reduce

trade_count < 50 OR confidence < 0.40 OR OOS ratio < 0.70
    ──► Research

health >= 80 AND expectancy > 0.15 AND pf >= 2.0 AND stability >= 0.75
    ──► IncreaseAllocation

default
    ──► Continue
```

---

## Performance Characteristics

| Metric     | Target | Achieved by                             |
|------------|--------|-----------------------------------------|
| Avg latency | < 3ms  | Pure Decimal arithmetic, no I/O        |
| P99 latency | < 10ms | No locks in hot paths                  |
| Allocations | Bounded| Vec only from caller-provided inputs    |
| Determinism | 100%   | Verified by 100,000-iteration MC tests  |
