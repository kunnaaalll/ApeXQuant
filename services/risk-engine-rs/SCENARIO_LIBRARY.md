# SCENARIO LIBRARY

This module contains predefined conditions describing historical and synthetic shocks.

## Predefined Scenarios
- `FlashCrash`
- `BlackMonday1987`
- `CovidCrash2020`
- `DotComBubble`
- `Lehman2008`
- `SyntheticExtreme`

## Behaviors
Each scenario impacts four separate risk vectors with calibrated multipliers:
1. **Volatility Multiplier:** Scales asset price standard deviations significantly.
2. **Correlation Multiplier:** Shifts the baseline matrix rapidly toward full positive correlation.
3. **Liquidity Reduction:** Radically diminishes order book depth and artificially increases slippage.
4. **Leverage Amplification:** Reflects forced liquidations causing systemic deleveraging spirals.
