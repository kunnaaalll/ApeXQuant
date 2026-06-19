# Risk Engine Shadow Mode Implementation

## Overview
Shadow Mode validates `risk-engine-rs` against the legacy implementation without executing trades. It guarantees that the Rust engine replicates the existing decision framework correctly over a period before Go-Live.

## Components
- **ComparisonEngine**: Matches qualitative and quantitative fields using safe thresholds.
- **DriftEngine**: Calculates absolute and relative deviations of numerical factors (Drawdown, Exposure, etc.).
- **StatisticsEngine**: Tracks comparison metrics and computes agreement percentages within daily/weekly/monthly buckets.
- **GoLiveValidator**: Manages state transitions (NotReady -> Monitoring -> Candidate -> Approved) exclusively through uninterrupted match streaks.
- **Reporter**: Serializes results into Markdown and JSON forms.
- **ShadowStorage**: Provides an append-only persistence trait for replayability.
