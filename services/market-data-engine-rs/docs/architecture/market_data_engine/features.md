# Feature Extraction

The features module serves as the foundational data layer for downstream intelligence logic. It computes rolling statistics and standardizations.

## Components

- **Rolling Calculations**: Efficient SMA (Simple Moving Average), EMA (Exponential Moving Average), and RMA calculations without buffer overruns.
- **Normalization**: Z-score calculation mechanisms.
- **Limits and Bounds**: Absolute limits are applied immediately to standard features to prevent unbounded mathematical blowups in later engine stages.
