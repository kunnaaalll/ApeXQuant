# Clustering Engine Implementation

## Overview
The `clustering` module dynamically categorizes strategy market conditions into predefined clusters based on bounded confidence matrices.

## Supported Clusters
- RiskOn
- RiskOff
- Momentum
- Breakout
- TrendFollowing
- Scalping
- Swing
- MeanReversion

## Design Features
- Deterministic event sourcing via `ClusterEvent` and `ClusterSnapshot`.
- Confidence bounds clamped explicitly between 0.0 to 100.0.
