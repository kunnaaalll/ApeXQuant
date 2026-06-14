# APEX V3 Portfolio Engine Validation Tests

## Overview
Test suite design for institutional-grade reliability.

## 1. Unit Tests
- Isolated testing of all state transition rules.
- Boundary condition testing for capital allocation margins and reserve depletion.

## 2. Integration Tests
- Validating End-to-End processing: `Event -> Allocator -> Evaluator -> Snapshots`.
- Ensuring proper interactions with the Event Store and Redis cache.

## 3. Replay Tests
- Load 1..N historical events.
- Reconstruct the `PortfolioState` dynamically.
- Assert snapshot hash exactly equals historical known hashes.

## 4. Property Tests
- Fuzzing all numerical constraints (e.g., limits, position sizes, allocation percentages).
- Ensuring non-negative capital, zero out-of-bounds exposure, and exact float summation precision.

## 5. Stress Tests
- Simulate 1,000,000 rapid sequential events.
- Disconnect database mid-execution and assert the circuit breaker logic handles failure without a process panic.

## 6. Monte Carlo Tests
- 10,000 random portfolio states run through drawdown scenarios.
- Require >99% survival rate across randomized events.

## 7. Determinism Tests
- 100,000 concurrent state updates using the same exact random seed.
- Verify exact state identical across all runs.
