# PORTFOLIO ANALYTICS IMPLEMENTATION

This document defines the implementation philosophy and specific metric definitions for the APEX V3 Portfolio Analytics Engine. Analytics are strictly descriptive and **never** make operational decisions. They answer the question: *"How well are we actually performing?"*

## Metric Definitions

1. **Expectancy**: Expected average return of a single trade based on historical win rate, average win, and average loss.
2. **Profit Factor**: Gross profit divided by gross loss. (APEX Invariant: >= 0)
3. **Sharpe Ratio**: Average return earned in excess of the risk-free rate per unit of volatility.
4. **Sortino Ratio**: Modification of Sharpe differentiating harmful volatility from overall volatility (using downside deviation).
5. **Calmar Ratio**: Annualized return divided by maximum drawdown.
6. **Recovery Factor**: Total net profit divided by maximum drawdown.
7. **Ulcer Index**: Measure of the depth and duration of drawdowns.

## Efficiency Philosophy

Efficiency metrics determine how well capital is being utilized over time:
- **Capital Efficiency**: Return generated per unit of capital deployed.
- **Allocation Efficiency**: Returns relative to the optimal sizing of trades.
- **Holding Efficiency**: Returns relative to the duration the trade was held open.
- **Risk Efficiency**: Returns relative to the initial stop loss distance.
- **Recovery Efficiency**: Speed at which equity recovers to new highs following a drawdown.

## Regime Performance Philosophy

The market behaves in distinct regimes. By isolating performance across these regimes, APEX identifies where edges are strongest:
- **Trending vs. Ranging**: Evaluated using directional volume and price volatility.
- **Volatility States**: High vs. Low volatility performance determines sensitivity to market pace.
- **Session Trading**: London, New York, and Asia session performance segmentation to detect liquidity preferences.

## Known Limitations

- **Decay Windows**: Rolling expectancy and time under water currently rely on static windows, meaning abrupt market regime shifts may have a delayed reflection in the trailing metrics.
- **Outlier Distortion**: Massive singular liquidation events could distort the standard deviation models if not explicitly clipped.
