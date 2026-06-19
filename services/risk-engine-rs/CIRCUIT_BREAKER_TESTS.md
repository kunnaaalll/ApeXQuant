# Circuit Breaker Testing Methodology

## Overview
The `circuit_breaker` module relies entirely on property-based and deterministic state-machine tests. Randomness is expressly forbidden in testing these core systems to ensure that a CI pipeline running 1,000 times will output identical results 1,000 times.

## Verified Properties

### 1. State Transitions
Tests explicitly forbid invalid transitions. Attempting to transition from `Frozen -> Normal` yields an `InvalidTransition` error. The `test_forbidden_transitions` case validates this boundary. Recovery must strictly pass through consecutive intermediate stages.

### 2. Drawdown & Leverage Bounds
Tests enforce that capacity bounds mathematically cannot cross zero into negative territory. `DrawdownLimitAssessment` and `LeverageAssessment` clamp negative outputs to `Decimal::ZERO` automatically.

### 3. Severity Clamping
The `overall_risk_restriction_score` is strictly bound between `0` and `100`. The test `test_severity_clamp` ensures that passing an extreme value like `150` normalizes to `100`, while negative values clamp to `0`.

### 4. Cooldown and Recovery Dynamics
The `test_cooldown_behavior` and `test_recovery_decay` evaluate time-based state restoration. Any loss injected into the system instantly resets cooldown counters and triggers an exponential decay in recovery progress.

### 5. 100,000 Iteration Determinism
The engine's stability over extended periods is proven via `test_determinism_100k_iterations`. The test simulates 100,000 sequential state updates. It validates that internal integers and states behave completely predictably with zero drift or divergence.

### 6. Event Replay
`test_event_replay` validates the Event Sourcing architecture. It takes a raw vector of `CircuitBreakerEvent`s, reconstructs a `CircuitBreakerSnapshot` from an initial epoch, and compares internal state matching exactly with the expected values.
