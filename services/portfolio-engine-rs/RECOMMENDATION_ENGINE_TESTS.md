# Recommendation Engine Tests

## Overview
The Recommendation Subsystem is rigorously tested using multiple methodologies to ensure absolute correctness and zero-panic behavior.

## Unit Tests
- Individual engines (`IncreaseExposureEngine`, `ReduceExposureEngine`, `CloseExposureEngine`, `BlockEngine`) are tested against specific input scenarios (Heat boundaries, Health boundaries, Drawdown flags).
- Verifies proper enum outcomes and populated explanations.

## Consistency Tests
- Verifies that the `RecommendationConsistencyValidator` accurately catches contradictions, ensuring the engines never recommend mutually exclusive actions (e.g., freezing the portfolio but simultaneously recommending increasing exposure).

## Property Tests (Fuzz Testing)
- The engines are tested against randomized `heat`, `health`, and `quality` scores (`0-100`) alongside boolean `is_frozen` and `is_critical_drawdown` flags.
- **Goals**:
  - Guarantee zero panics under any combinations.
  - Guarantee deterministic output for specific seeds.
  - Guarantee no unhandled scenarios.

## Snapshot & Determinism Tests
- Checks if the `RecommendationSnapshot` and `RecommendationEvent` structures correctly serialize and deserialize.
- Verifies that identical input states always generate identical snapshots.
