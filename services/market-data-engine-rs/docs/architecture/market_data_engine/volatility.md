# Volatility Engine

The Volatility engine calculates current realized volatility across custom periods.

## Components

- **ATR Engine**: Average True Range is calculated safely. High-low spreads and gap risks are factored in.
- **Expanding / Contracting State**: Detects volatility squeezes and breakouts using historical variance comparisons.
- **Volatility Score (0-100)**: Normalizes volatility against typical baseline bounds to give a percentage score for market risk.
