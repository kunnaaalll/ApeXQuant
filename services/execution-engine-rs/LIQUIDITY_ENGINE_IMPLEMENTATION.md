# Liquidity Engine Implementation

Evaluates venue liquidity using bounded, deterministic scoring out of 100.

## Regime
Classifies overall market state into: `Illiquid`, `Weak`, `Normal`, `Healthy`, `Excellent`.

## Depth Score
Calculates available vs required depth. Scores 100 if sufficient, degrades linearly if not.

## Spread Quality
Compares current spread vs historical. 100 if equal or better, degrading to 0 if 2x historical.

## Imbalance
Calculates Order Book Imbalance from -1.0 (sell pressure) to 1.0 (buy pressure).
