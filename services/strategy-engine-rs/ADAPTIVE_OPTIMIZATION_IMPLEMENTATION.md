# Adaptive Optimization Implementation

## Overview
The `adaptive` module provides adaptive optimization intelligence for APEX Strategy Engine, incorporating deterministic EMA decay models and weight optimizers for Symbol, Regime, Session, Timeframe, and Pattern strategies.

## Invariants
- 100% Deterministic (Zero floating-point arithmetic).
- Zero panics or unwraps.
- Weights bound strictly between 0.50 and 2.00.
- Maximum shift per cycle strictly limited to 0.05.

## Components
- `DecayModel`: Smooths signals using EMA with a clamped alpha value.
- `WeightOptimizer`: Adaptively increments or decrements strategy weights safely without jumping abruptly.
- `AdaptiveState`: Immutable and snapshot-capable state representation.
- `AdaptiveEvent`: Event-sourcing variants.
