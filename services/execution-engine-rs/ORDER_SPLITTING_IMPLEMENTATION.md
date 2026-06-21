# Order Splitting Implementation

Orders are split deterministically without random bounds to ensure replayability.

## TWAP (Time-Weighted Average Price)
Splits order equally into N `slices`.
The last slice absorbs the remainder to prevent rounding losses.

## VWAP (Volume-Weighted Average Price)
Splits order according to an array of proportional weights.
Weights dictate what fraction of the total quantity is placed in each slice.

## Iceberg
Maintains a `visible_quantity`. Slices the total quantity into increments of exactly `visible_quantity`, with the final slice taking the remaining hidden quantity.

## Allocator
Routes child order fills back to parent logic and aggregates child fill outcomes.
