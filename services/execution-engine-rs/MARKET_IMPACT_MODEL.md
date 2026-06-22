# MARKET IMPACT MODEL

## Overview
Provides a bounded estimate of market impact resulting from execution slippage, transforming raw slippage in bps to a discrete grade and normalized score.

## Grades
- `Negligible`: <= 1 bps
- `Low`: <= 5 bps
- `Moderate`: <= 15 bps
- `High`: <= 30 bps
- `Extreme`: > 30 bps

## Execution Integration
The impact model computes a `0-100` penalty score which directly factors into the Total Execution Cost score.
