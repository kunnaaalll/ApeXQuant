# EXECUTION COST MODEL

## Overview
Computes a comprehensive cost of execution integrating spread, slippage, and impact costs calculated deterministically using `rust_decimal::Decimal`.

## Components
- **SpreadCost**: Notional * spread bps
- **SlippageCost**: Notional * slippage bps
- **ImpactCost**: Notional * impact bps

## Grade Mapping
Calculates an aggregate cost in bps against the notional to produce a discrete Grade:
- `Excellent`: <= 1 bps
- `Good`: <= 5 bps
- `Average`: <= 15 bps
- `Poor`: <= 30 bps
- `Extreme`: > 30 bps
