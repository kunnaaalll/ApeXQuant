# Discovery Engine Implementation

## Overview
The `discovery` engine identifies changing edge parameters, tracks metric velocities, and catches deterioration early to facilitate defensive interventions. 

## Capabilities
- **EdgeDiscovery**: Evaluates `edge_delta`, `expectancy_delta`, and `confidence_delta` to yield states like Emerging, Strengthening, Stable, Weakening, or Collapsing.
- **VelocityEngine**: Tracks momentum of core metrics across cycles to identify Accelerating, Stable, Decelerating, or Reversing trends.
- **DeteriorationDetector**: Evaluates drawdown increases and expectancy collapses. Can issue immediate critical warnings while requiring gradual recovery.

## Invariants
- Zero float execution.
- Deterministic detection conditions.
- Strict bounded variables.
