# LATENCY ENGINE

## Overview
Monitors latencies deterministically to detect network degradation and calculate a bounded health score.

## Components
1. **Latency Measurement**: Tracks `broker`, `exchange`, and `network` latencies in ms.
2. **Degradation Detector**: Flags if current total latency exceeds 200% of baseline and is greater than 50ms.
3. **Health State**:
   - `Excellent`: <= 10ms
   - `Healthy`: <= 50ms
   - `Warning`: <= 150ms
   - `Critical`: > 150ms

## Penalties
Calculates a 0-100 penalty scaled linearly until 200ms, effectively saturating to 0 score.
