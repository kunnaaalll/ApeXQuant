# Stability Module Implementation

## Overview
The Stability module measures risk-adjusted performance, drawdowns, and variance. It determines if the portfolio's edge is robust or subject to extreme volatility.

## Core Metrics
- **Sharpe Ratio:** Mean return per unit of volatility.
- **Sortino Ratio:** Mean return per unit of downside volatility.
- **Calmar Ratio:** Annualized return over maximum drawdown.
- **Ulcer Index:** Measure of depth and duration of drawdowns.
- **Recovery Factor:** Net profit divided by maximum drawdown.
- **Consistency:** Percentage of profitable sub-periods.
- **Variance:** Variance of returns.
- **Stability Score:** Normalized 0-100 composite stability score.

## State Transitions
- **Excellent:** Highly stable, high risk-adjusted returns.
- **Strong:** Consistently profitable, manageable variance.
- **Stable:** Baseline stability, acceptable drawdowns.
- **Weak:** High variance, long recovery times.
- **Critical:** Unstable, massive drawdowns, high risk of ruin.

## Safe Math Guarantees
- Volatility measurements that yield 0 (e.g. flat returns) result in bounded fallback ratio values.
- Ratios are capped (e.g. max Sharpe of 100.0) to prevent `Infinity` equivalents in `Decimal`.
