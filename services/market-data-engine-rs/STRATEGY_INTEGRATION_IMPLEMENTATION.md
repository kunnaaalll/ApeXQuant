# Strategy Integration Implementation

## Overview
The Strategy Integration layer provides adapters that package continuous market intelligence for ingestion by `strategy-engine-rs`.

## Key Outputs
- **StrategyIntelligenceSnapshot**: Complete view of a symbol's strategic landscape at a point in time.
- **RegimeSnapshot**: Probability distribution of the current market regime (e.g., Trending, Mean-Reverting).
- **OpportunitySnapshot**: Quantified expected value and scoring for strategy signal generation.
- **SymbolHealthSnapshot**: Data quality gating signals to prevent strategy misfire.
- **ConfidenceInputs**: Model and market confidence scores for position sizing.
