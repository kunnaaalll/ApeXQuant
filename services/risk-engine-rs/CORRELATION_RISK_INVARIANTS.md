# Correlation Risk Invariants

These invariants are strictly enforced via the `Decimal` crate and compiler rules.

## Strict Mathematical Invariants

-   **Bounded Correlation**: `correlation ∈ [-1.0, 1.0]`
-   **Hidden Leverage Base**: `hidden_leverage_ratio >= 0`
-   **Cluster Definitions**: `cluster_weight >= 0`, `cluster_concentration ∈ [0, 100]`

## Severity Thresholds

Severity levels do not skip states. Escalation pathways strictly follow:
1.  **Low**
2.  **Moderate**
3.  **Elevated**
4.  **High**
5.  **Critical**
6.  **Collapse**

### Hidden Leverage Triggers
-   `>= 1.2` -> Elevated
-   `>= 2.0` -> High
-   `>= 3.0` -> Critical
-   `>= 4.0` -> Collapse

## Compiler Restrictions

-   `#![deny(unsafe_code)]` is strictly mandated.
-   No `f64` or `f32` types are allowed for risk matrix states.
-   `unwrap()` and `expect()` are strongly discouraged during matrix access.
