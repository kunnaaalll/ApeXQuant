# Trend Engine

The Trend engine scores directional market moves based on both short-term and long-term EMA intersections.

## Logic

- Tracks short-term (e.g. 10-period) vs long-term (e.g. 30-period) EMAs.
- Calculates moving average distances (spread between short/long).
- Computes `TrendState` (Uptrend, Downtrend, Sideways) and a normalized `strength_score` (0-100).
