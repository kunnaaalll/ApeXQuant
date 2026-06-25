# Portfolio Integration Implementation

## Overview
The Portfolio Integration layer surfaces macro-level features to `portfolio-engine-rs` to assist in cross-asset allocation and risk-adjusted capital deployment.

## Key Outputs
- **AssetCorrelationMatrix**: Deeply nested dynamic correlations between all tradeable assets.
- **DiversificationScore**: A single aggregate score denoting the current state of market co-movement.
- **ExposureBuckets**: Thematic or sector-based exposure groupings.
- **HeatSignals**: Real-time identification of assets that are rapidly absorbing or losing liquidity.
- **AllocationSuggestions**: Recommended capital weight shifts driven by structural regime changes.
