# APEX Risk Engine V1

## Overview
The V1 Risk Engine provides deterministic, panic-free, institutional-grade risk management for the APEX trading system. It operates completely on immutable snapshots and event sourcing principles, utilizing exclusively `rust_decimal` for financial calculations. 

## Features
- **Value at Risk (VaR):** Evaluates portfolio risk states based on calculated thresholds.
- **Conditional VaR (CVaR):** Provides deeper tail loss insights with explicit Extreme, Dangerous, and Elevated states.
- **Kelly Engine:** Optimizes allocation fractions using half-Kelly adjustments, strictly bounded to `0 -> 0.5`.
- **Drawdown Risk:** Limits exposure progressively as the portfolio approaches max drawdown thresholds.
- **Tail Risk:** Accounts for Black Swan events and collapse probability.
- **Stress & Scenario Engines:** Runs multiple crisis simulations and probabilistic path assessments.

## Certification
- Zero panics (`#![deny(clippy::panic)]`, `#![deny(clippy::unwrap_used)]`).
- Zero unsafe code (`#![deny(unsafe_code)]`).
- `rust_decimal` exclusively.
- Deterministic only.
