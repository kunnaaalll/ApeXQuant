# Regime Engine

The Regime engine fuses inputs from Volatility, Trend, and Structure to deduce the broader macro-environmental state of the market.

## Matrix

Maps combinations of volatility expansion/contraction, trend strength, and structural breakouts into:
- `Trending` (high trend, expanding structure)
- `MeanReverting` (low trend, contracting volatility)
- `Breakout` (high volatility expansion, breaking structure limits)
- `Chaotic` (high noise, unpredictable behavior)

Includes a confidence score indicating the clarity of the regime signal.
