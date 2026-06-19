# Context Ranking Implementation

## Overview
Context Ranking produces a normalized, bounded composite score and Tier to evaluate strategy contexts uniformly.

## Calculation
`expectancy * confidence * stability / max(drawdown, 0.0001)`
Ensures zero panic on division by zero. Bounds grades from `Forbidden` to `Elite`.
