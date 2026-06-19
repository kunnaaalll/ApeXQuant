# Recommendation Engine Implementation

## Overview
The `recommendation` module distills complex metric combinations into actionable, deterministic advisory recommendations.

## Core Rules
- **Non-Executing**: Recommendations are advisory only. They **never** execute trades directly.
- **Determinism**: Outputs rely purely on Decimal arithmetic over bounded inputs.

## Engine Behavior
It processes `edge_strength`, `risk_level`, and `stability` to output an `action` alongside precise `reason_codes`.

## Actions
- Increase
- Maintain
- Reduce
- Pause
- Research
- Retire

## Reason Codes
Includes explicitly mapped indicators like `EdgeEmerging`, `ExcessiveRisk`, `PoorSampleQuality`, and `ExcellentStability`.
