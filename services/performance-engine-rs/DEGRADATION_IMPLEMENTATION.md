# Degradation Module Implementation

## Overview
The Degradation module detects structural drift, edge decay, and expectancy deterioration. It acts as the "canary in the coal mine" for strategy obsolescence.

## Core Metrics
- **Edge Decay:** Rate of edge reduction.
- **Expectancy Decay:** Rate of expectancy reduction.
- **Quality Deterioration:** Degradation in trade quality scores.
- **Stability Deterioration:** Increase in variance/drawdowns.
- **Performance Drift:** Drift from historical mean expectancy.
- **Duration:** Time since degradation began.
- **Severity:** Magnitude of the degradation.
- **Velocity:** Rate of acceleration of the degradation.

## State Transitions
- **Healthy:** No significant degradation detected.
- **Watch:** Early signs of decay, requires monitoring.
- **Warning:** Statistically significant degradation present.
- **Critical:** Severe edge decay, strategy may be broken.

## Safe Math Guarantees
- Decay velocities bounded strictly using `rust_decimal` precision.
- No panics on zero history comparisons.
