# Slippage Engine Implementation

Calculates expected, realized, and market impact slippages.

## Expected
`Expected = Volatility * (Order_Size / Liquidity_Depth)`

## Realized
`Realized = Executed_Price - Expected_Price` (for buys)
`Realized = Expected_Price - Executed_Price` (for sells)

## Market Impact
Ratio approximation of impact based on order size vs ADV (Average Daily Volume) without using floating-point math functions like square root.

## Score
Scores Realized vs Max Acceptable Slippage from 100 (Perfect) down to 0 (Failed/Exceeded).
