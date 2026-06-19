# Risk Explainability Implementation

## Overview
The Explainability Layer transforms deterministic quantitative states into natural language risk narratives.

## Components
- **Reasons**: List of current inputs, ranked by severity.
- **Contributors**: Identifies the primary factor driving the risk policy.
- **Improvements**: Detects reduced exposures or VaR since the last state.
- **Deteriorations**: Detects rising correlation, tail risks, or drawdowns.
- **Constraints**: Formulates the rationale for why a more aggressive recommendation was rejected (e.g., "High correlation prevented aggressive sizing").
- **Summary**: Produces a `RiskNarrative` containing all elements to be logged and attached to the event.
