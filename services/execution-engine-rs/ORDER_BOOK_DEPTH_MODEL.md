# ORDER BOOK DEPTH MODEL

## Overview
Models cumulative liquidity across top book levels to categorize the strength of the market depth.

## Depth Grades
- `Excellent`: > 1000 notional equivalent
- `Strong`: > 500
- `Normal`: > 100
- `Weak`: > 10
- `Thin`: < 10

## Data Structure
Tracks `level1`, `level2`, `level3` volumes using `rust_decimal::Decimal` and evaluates `cumulative_depth`.

## Determinism
All evaluations yield discrete, well-defined enum states. No floating point logic is used to establish boundaries.
