# Expectancy Module Implementation

## Overview
The Expectancy module calculates the expected return of trades over various time horizons (daily, weekly, monthly, rolling). It provides a deterministic mathematical foundation using `rust_decimal::Decimal` for exact precision and safe math without panics.

## Core Metrics
- **Wins/Losses/Breakevens:** Absolute counts.
- **Average Win/Loss:** Arithmetic mean of winning and losing trades respectively.
- **Expectancy:** The mathematical expectation of a trade. `(Win Rate * Average Win) - (Loss Rate * Average Loss)`.
- **Profit Factor:** `Gross Profit / Gross Loss` (with bounds/safeguards against 0).
- **Average RR:** Reward-to-Risk ratio on a per-trade average.
- **Trade Count:** Total population size.

## State Transitions
- **Exceptional:** High expectancy, robust sample size.
- **Strong:** Consistently positive expectancy above baseline.
- **Normal:** Baseline positive expectancy.
- **Weak:** Marginally positive to flat, potentially degrading.
- **Negative:** Statistically negative expectancy.

## Safe Math Guarantees
- `rust_decimal` used universally.
- All division operations explicitly check for zero denominators. 
- Returns fallback zero values or explicitly defined bounds (e.g., maximum Decimal) on division-by-zero to prevent runtime panics.

## Events & Storage
- `ExpectancyEvent::Updated(Metrics)`: Appended to the event store.
- `ExpectancySnapshot`: Point-in-time deterministic reconstruction.
