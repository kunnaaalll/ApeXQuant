import os

files = [
    "MARKET_INTELLIGENCE_IMPLEMENTATION.md",
    "SPREAD_ENGINE.md",
    "VOLATILITY_ENGINE.md",
    "DEPTH_ENGINE.md",
    "IMBALANCE_ENGINE.md",
    "LIQUIDITY_ENGINE.md",
    "TREND_ENGINE.md",
    "MOMENTUM_ENGINE.md",
    "SESSION_ENGINE.md",
    "REGIME_ENGINE.md",
    "EFFICIENCY_ENGINE.md",
    "NOISE_ENGINE.md",
    "MARKET_QUALITY_ENGINE.md",
    "MARKET_CONFIDENCE_ENGINE.md",
    "ANOMALY_ENGINE.md",
    "MARKET_SCORE_MODEL.md",
    "MARKET_STATE_MACHINE.md",
    "INTELLIGENCE_TESTS.md",
    "INTELLIGENCE_INVARIANTS.md",
    "PHASE4_WALKTHROUGH.md"
]

template = """# {title}

This document outlines the determinism requirements and architecture for the {title} module in APEX V3 Phase 4.

## Constraints
- No floating-point types (`f32`/`f64`).
- No `unsafe` code.
- No `unwrap()` or `expect()`.
- Zero side effects.
"""

for file in files:
    title = file.replace(".md", "").replace("_", " ").title()
    with open(file, "w") as f:
        f.write(template.format(title=title))

print("Markdown documentation generated.")
