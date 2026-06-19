# Circuit Breakers and Dynamic Risk Limits (Phase 6)

## Overview
Phase 6 introduces an institutional-grade active defense mechanism known as the Circuit Breaker Engine. Unlike previous risk models that merely measured exposure or detected hidden leverage, the Circuit Breaker actively restricts, freezes, or halts risk-taking operations. 

It is designed to act as a fully deterministic "Risk Committee" that intervenes during:
- Tail risk events
- Drawdown capacity breaches
- Liquidity collapse
- Volatility spikes
- Hidden leverage accumulation

## Architecture

The system comprises 12 highly specialized deterministic engines:
1. **Circuit Breaker State Machine**: `Normal`, `Warning`, `Restricted`, `Critical`, `Frozen`.
2. **Trading Halt Engine**: Handles `SlowMode`, `Blocked`, and `EmergencyStop`.
3. **Risk Limits**: Daily, weekly, monthly limits with active capacity tracking.
4. **Drawdown Limit**: Tracks maximum allowable drawdown before freezing.
5. **Exposure Limit**: Symbol, currency, and theme concentration bounds.
6. **Leverage Limit**: Gross, effective, and hidden leverage checks.
7. **Liquidity Limit**: Execution degradation and spread limits.
8. **Volatility Limit**: Bounded volatility multiplier constraints.
9. **Cooldown Model**: Required downtime after risk spikes before resuming operations.
10. **Recovery Model**: Exponentially decaying recovery that resets upon successive losses.
11. **Escalation Model**: Dynamic risk scoring across multiple dimensions.
12. **Severity Model**: A 0-100 normalization engine.

## Determinism & State Replay
The engine is 100% deterministic, containing no randomness and zero floating-point math. All events are tracked in `CircuitBreakerSnapshot`s and replayable using the `CircuitBreakerEvent` log, enabling post-mortem audits that guarantee 1:1 state recreation.
