# Edge Discovery Implementation

## Overview
The `discovery` module seeks out migrating edge behavior by continuously tracking entity deterioration and improvement, serving as an early-warning system.

## Core Modules
- `edge_discovery.rs`: Tracks and categorizes entities (symbols, regimes, patterns, sessions) into discrete states: `Emerging`, `Stable`, `Weakening`, `Collapsing`. Classification is derived from the difference between short-term and long-term expectancy.
- `opportunity_detector.rs`: Identifies contexts becoming stronger (`StrongOpportunity`, `ModerateOpportunity`). It prevents false positives by enforcing a minimum `confidence` threshold.
- `deterioration_detector.rs`: The critical defensive layer. It detects edge collapse instantly when a drawdown ceiling is breached or expectancy decays past acceptable thresholds, outputting absolute state signals (`Healthy` to `Collapse`).

## Safety Guarantees
Edge discovery never executes trades. It deterministically feeds the `RecommendationEngine`. Thresholds and states are explicit; there are no probabilistic state transitions.
