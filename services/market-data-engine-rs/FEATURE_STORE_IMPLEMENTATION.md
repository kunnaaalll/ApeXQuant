# Feature Store Implementation

## Overview
The Feature Store serves as the deterministic repository for market data features, consolidating inputs for analytics and ML pipelines.

## Architecture
- **FeatureVector**: Strongly-typed storage for normalized feature states, including:
  - ATR
  - Realized Volatility
  - Spread
  - Momentum
  - EMA Distance
  - Correlation
  - Regime
  - Session
  - Market Quality
  - Trend Strength
  - Breakout State
- **FeatureSnapshot**: Represents the complete feature state of a symbol at a specific point in time and timeframe window.
- **FeatureWindow**: Standardized aggregation horizons (1m, 5m, 15m, 1h, 4h, Daily).
- **Store**: An in-memory concurrent mapping of symbol + window -> snapshot, supported by Postgres-based persistence (`FeatureStoreRepository`).
