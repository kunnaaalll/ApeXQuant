# Allocation Engine Implementation

## Overview
The `allocation` engine transforms strategy performance characteristics into definitive exposure state multipliers, dynamically governing deployed risk.

## Mechanism
Inputs: `strategy_health`, `confidence`, `degradation`, `drawdown`, `sample_quality`.
The engine processes a comprehensive risk vs safety computation to determine the exposure phase.

## Exposure States
- `IncreaseExposure`
- `SlightIncrease`
- `Maintain`
- `ReduceExposure`
- `Pause`
- `Block`

## Multiplier Bounds
Multipliers are dynamically calculated but explicitly restricted to a 0.25x -> 2.00x range, eliminating uncontrollable scaling behaviors.
