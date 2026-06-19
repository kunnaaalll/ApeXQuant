# Risk Recommendation Engine Implementation

## Overview
Phase 7 introduces the institutional decision layer (Risk Committee).
The system translates individual risk engine outputs (drawdown, VaR, correlation) into actionable deterministic states.

## Architecture

1. **Increase Engine**: Determines conditions allowing for capital/leverage increase.
2. **Reduction Engine**: Maps negative risk developments to varying degrees of capital reduction.
3. **Block Engine**: Dictates the Trade Admission Policy (Freeze, Block, Delay, Allow).
4. **Freeze Engine**: Highest priority circuit breaker forcing a global halt.
5. **Risk Committee**: Aggregates all engine outputs sequentially based on the strict hierarchy.

## Determinism
All outputs are fully deterministic. We enforce zero floating-point arithmetic and no machine learning models.
