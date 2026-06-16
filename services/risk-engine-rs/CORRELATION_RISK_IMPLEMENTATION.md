# Correlation Risk Engine Implementation

The Correlation Risk Engine (Phase 3) extends the APEX risk capabilities by introducing advanced correlation discovery and hidden leverage assessment. This ensures that the engine is capable of detecting non-obvious concentrations, overlapping exposures, and systemic risks across dimensions.

## Mathematical Philosophy

At its core, the engine models correlations as a bounded \([-1.0, 1.0]\) metric. To maintain strict institutional-grade determinism, we use `rust_decimal::Decimal` and eliminate all floating point (`f32`/`f64`) properties. 

```rust
let pos_one = Decimal::new(1, 0);
let neg_one = Decimal::new(-1, 0);
```

### Overlap Logic
When calculating overlap between portfolios or symbols, the Engine aggregates the correlation matrices to establish:

1.  **Synthetic Duplication**: Detected when two seemingly distinct instruments exhibit `correlation > 0.95`.
2.  **Directional Overlap**: Overweight beta exposure to the same factor.
3.  **Currency Overlap**: Hidden exposure to a single fiat unit (e.g., USD, EUR).
4.  **Theme/Sector Overlap**: Aggregation of distinct securities that belong to overlapping themes (e.g., Tech and Crypto mapping to Risk-On).

### Hidden Leverage Theory
Hidden leverage exists when positions mimic the returns of an identical underlying factor. Instead of solely looking at gross position size, the `HiddenLeverageAssessment` weights overlaps to construct a `total_hidden_leverage_ratio`. If the ratio breaches institutional thresholds (e.g., `>= 4.0`), the system escalates the state to `Collapse`.

### Cluster Philosophy
Clusters categorize assets not by explicit tags, but by their real-time correlation bounds. We categorize into:
-   **RiskOn**: Crypto, Tech, Growth
-   **RiskOff**: Bonds, Metals, Defensive
-   **Inflation**
-   **Commodity**

A cluster's severity depends on its weight and concentration bounds `[0, 100]`.
