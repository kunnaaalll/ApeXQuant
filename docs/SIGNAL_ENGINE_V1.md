# Signal Engine V1

## Overview

The Signal Engine is a deterministic, institutional-grade signal generation service for intraday swing trading. It analyzes market structure, multi-timeframe alignment, and Smart Money Concepts (SMC) to produce high-quality trade opportunities.

**Trade Style:** Intraday swing trading (30 minutes to 5 hours)  
**Quality Threshold:** A+ and A signals only (B signals logged, not emitted)  
**Target Risk:Reward:** 1:2 minimum, prefer 1:3 to 1:4

## Architecture

```
services/signal-engine-rs/
├── Cargo.toml
└── src/
    ├── main.rs              # Service entry point
    ├── lib.rs               # Public API
    ├── config.rs            # Configuration management
    ├── error.rs             # Error types
    ├── metrics.rs           # Prometheus instrumentation
    ├── health.rs            # Health check endpoints
    │
    ├── market_data/         # Market data ingestion
    │   ├── mod.rs
    │   ├── candle.rs        # OHLCV representation
    │   ├── buffer.rs        # Timeframe buffers
    │   └── validator.rs     # Data quality checks
    │
    ├── regime/              # Market regime detection
    │   ├── mod.rs
    │   ├── detector.rs      # Regime classification
    │   ├── types.rs         # RegimeType enum
    │   └── volatility.rs    # Volatility calculations
    │
    ├── mtf/                 # Multi-timeframe analysis
    │   ├── mod.rs
    │   ├── aligner.rs       # Cross-timeframe alignment
    │   ├── hierarchy.rs     # H4 → H1 → M30 → M15
    │   └── types.rs         # MTFAlignmentResult
    │
    ├── smc/                 # Smart Money Concepts
    │   ├── mod.rs
    │   ├── bos.rs           # Break of Structure
    │   ├── choch.rs         # Change of Character
    │   ├── order_blocks.rs  # Order block detection
    │   ├── fvg.rs           # Fair Value Gaps
    │   ├── liquidity.rs     # Liquidity sweeps
    │   ├── displacement.rs  # Displacement detection
    │   ├── mitigation.rs    # Mitigation tracking
    │   ├── imbalance.rs     # Imbalance zones
    │   └── premium_discount.rs  # Premium/Discount zones
    │
    ├── structure/           # Market structure analysis
    │   ├── mod.rs
    │   ├── swings.rs        # Swing high/low detection
    │   ├── trend.rs         # Trend detection
    │   ├── ranges.rs        # Range identification
    │   ├── impulse.rs       # Impulse wave detection
    │   └── correction.rs    # Correction detection
    │
    ├── confluence/          # Confluence scoring engine
    │   ├── mod.rs
    │   ├── engine.rs        # Weighted scoring logic
    │   ├── factors.rs       # Individual factor definitions
    │   └── weights.rs       # Dynamic weight adjustment
    │
    ├── scoring/             # Signal scoring
    │   ├── mod.rs
    │   ├── score.rs         # ConfluenceScore (0-100)
 │   ├── quality.rs         # Signal quality (A+, A, B, Reject)
    │   └── grade.rs         # Grading logic
    │
    ├── confidence/          # Confidence calculation
    │   ├── mod.rs
    │   ├── calculator.rs    # Confidence computation
    │   ├── factors.rs       # Confidence factors
    │   └── decay.rs         # Time-based decay
    │
    ├── filters/             # Signal filters
    │   ├── mod.rs
    │   ├── quality.rs       # Quality threshold filter
    │   ├── regime.rs        # Regime-based filtering
    │   ├── session.rs       # Session-based filtering
    │   └── duplicates.rs    # Duplicate suppression
    │
    ├── evidence/            # Explainability
    │   ├── mod.rs
    │   ├── collector.rs     # Evidence aggregation
    │   ├── builder.rs       # Reason construction
    │   └── formatter.rs     # Human-readable output
    │
    ├── signals/             # Signal orchestration
    │   ├── mod.rs
    │   ├── generator.rs     # Main signal generation
    │   ├── result.rs        # SignalResult struct
    │   ├── validator.rs     # Pre-emission validation
    │   └── emitter.rs       # Signal emission
    │
    └── api/                 # gRPC service implementation
        ├── mod.rs
        ├── server.rs        # Tonic gRPC server
        ├── service.rs       # SignalEngine service impl
        └── interceptors.rs  # Auth, logging, etc.
```

## Core Types

### SignalResult

```rust
pub struct SignalResult {
    // Identification
    pub signal_id: Uuid,
    pub symbol: Symbol,
    pub direction: TradeSide,

    // Timing
    pub detected_at: Timestamp,
    pub valid_until: Timestamp,
    pub timeframe: Timeframe,

    // Scoring
    pub confluence_score: ConfluenceScore,      // 0-100
    pub confidence: Confidence,                  // 0.0 - 1.0
    pub quality: SignalQuality,                  // A+, A, B, Reject

    // Market Context
    pub market_regime: MarketRegime,
    pub mtf_alignment: MTFAlignmentResult,

    // Price Levels
    pub entry_zone: PriceZone,
    pub stop_zone: PriceZone,
    pub target_zone: PriceZone,
    pub risk_reward: Decimal,

    // Evidence
    pub patterns_detected: Vec<Pattern>,
    pub evidence: EvidenceCollection,
    pub reasons: Vec<String>,

    // Metadata
    pub detector_version: String,
    pub metadata: SignalMetadata,
}
```

### ConfluenceScore

```rust
pub struct ConfluenceScore {
    pub total: u8,           // 0-100 composite score
    pub factors: Vec<ConfluenceFactor>,
}

pub struct ConfluenceFactor {
    pub factor_type: FactorType,
    pub weight: f64,
    pub contribution: f64,   // Raw contribution to score
    pub normalized_score: f64,
}

pub enum FactorType {
    HTFAlignment,
    Regime,
    Momentum,
    Liquidity,
    OrderBlock,
    FVG,
    Displacement,
    Session,
    StructureQuality,
    TrendStrength,
    Volatility,
}
```

### MarketRegime

```rust
pub struct MarketRegime {
    pub regime_type: RegimeType,
    pub confidence: f64,
    pub start_time: Timestamp,
    pub bars_in_regime: u32,
    pub volatility_percentile: f64,
    pub trend_strength: f64,
}

pub enum RegimeType {
    TrendingUp,
    TrendingDown,
    Ranging,
    Breakout,
    HighVolatility,
    LowVolatility,
    Transition,
}
```

### MTFAlignmentResult

```rust
pub struct MTFAlignmentResult {
    pub aligned: bool,
    pub reference_timeframe: Timeframe,
    pub alignments: Vec<TimeframeAlignment>,
    pub alignment_score: f64,
    pub bias: MarketBias,
}

pub struct TimeframeAlignment {
    pub timeframe: Timeframe,
    pub direction: AlignmentDirection,
    pub weight: f64,
    pub context: String,
}

pub enum AlignmentDirection {
    Bullish,
    Bearish,
    Neutral,
    Conflict,
}
```

## Modules

### Market Data (market_data/)

**Purpose:** Ingest and normalize OHLCV data from multiple sources.

**Key Components:**
- `Candle`: OHLCV with metadata (confirmed, partial)
- `Buffer`: Timeframe-specific circular buffers
- `Validator`: Data quality checks (gaps, outliers, staleness)

**Assumptions:**
- MT5 provides data via gRPC streaming
- Candles are confirmed on close of next candle
- Ticks provide sub-candle precision

### Market Structure (structure/)

**Purpose:** Identify foundational market structure elements.

**Key Components:**
- `SwingDetector`: Identifies swing highs/lows using pivot logic
- `TrendDetector`: Higher highs/lows vs lower highs/lows
- `RangeDetector`: Support/resistance bounds, range validity
- `ImpulseDetector`: Strong directional moves
- `CorrectionDetector`: Pullbacks within trend

**Algorithms:**
- Swing detection: Requires N bars on each side (default: 3)
- Trend: Structure-based, not just MA slope
- Ranges: 2+ touches on both sides, contained movement

### Multi-Timeframe Analysis (mtf/)

**Purpose:** Ensure lower timeframe signals align with higher timeframe bias.

**Hierarchy:**
```
H4 (Structure) → H1 (Bias) → M30 (Context) → M15 (Execution)
```

**Alignment Rules:**
- H4 bearish structure: Only consider sell signals on M15
- H1 bullish CHoCH: Buy bias on M30/M15
- Conflicting HTF: Reduce confidence, widen zones
- All aligned: Maximum confluence score

### Smart Money Concepts (smc/)

**Purpose:** Detect institutional order flow patterns.

#### BOS (Break of Structure)
- Bullish: Close above previous lower high
- Bearish: Close below previous higher low
- Must occur in context of established trend

#### CHoCH (Change of Character)
- Bullish: First higher high in downtrend
- Bearish: First lower low in uptrend
- Earlier signal than BOS, higher risk/reward

#### Order Blocks
- Bullish: Last bearish candle before bullish impulse
- Bearish: Last bullish candle before bearish impulse
- Must be mitigation-level for highest quality
- Track age: Recent (0-10 bars), aged (10-30), stale (>30)

#### Fair Value Gaps
- Bullish: Current low > previous high (bearish candle gap)
- Bearish: Current high < previous low (bullish candle gap)
- Measure fill percentage
- Classify: Unfilled, partially filled, filled, overfilled

#### Liquidity Sweeps
- Equal highs/lows taken out with reversal
- Stop hunting patterns
- Must show displacement after sweep

#### Displacement
- 3+ candles in direction, minimal wicks
- ATR multiple (default: 1.5x ATR)
- Indicates institutional participation

### Confluence Engine (confluence/)

**Purpose:** Probabilistic scoring instead of rigid rule chains.

**Design Philosophy:**
```
// WRONG - Rigid chain
if (bos && ob && fvg && htf_aligned) { signal }

// RIGHT - Probabilistic
score = bos.weight * bos.strength +
        ob.weight * ob.quality +
        fvg.weight * fvg.fill_ratio +
        htf.weight * htf.alignment_score
```

**Factor Weights (Default):**
| Factor | Weight | Notes |
|--------|--------|-------|
| HTF Alignment | 0.20 | Most important - directional bias |
| Regime Fit | 0.15 | Trending vs ranging |
| Momentum | 0.12 | RSI, volume slope |
| Liquidity | 0.12 | Proximity to key levels |
| Order Block | 0.10 | Quality and freshness |
| FVG | 0.08 | Fill status |
| Displacement | 0.08 | Strength of move |
| Session | 0.07 | Trading session context |
| Structure Quality | 0.05 | Clean vs choppy |
| Trend Strength | 0.03 | ADX, slope |

**Dynamic Weight Adjustment:**
- Increase HTF weight when regimes are clear
- Reduce OB weight when blocks are aged
- Increase liquidity weight near key levels
- Adjust for volatility regime

### Signal Quality (scoring/)

**Grading:**
| Grade | Min Score | Min R:R | Requirements |
|-------|-----------|---------|--------------|
| A+ | 85 | 1:3 | All HTF aligned, fresh patterns |
| A | 70 | 1:2 | Strong alignment, good setup |
| B | 60 | 1:1.5 | Reasonable, but not ideal |
| Reject | <60 | Any | Below quality threshold |

**Emission Rules:**
- Only A+ and A signals emitted
- B signals logged for analysis
- Rejects silently dropped (metrics only)

### Explainability (evidence/)

Every signal must answer:

1. **Why buy/sell?**
   - Directional bias from HTF
   - Pattern triggering entry
   - Confluence factors supporting

2. **Why now?**
   - Timeframe alignment
   - Pattern maturity
   - Market context

3. **Why not wait?**
   - Entry zone validity window
   - Pattern degradation risk
   - Regime stability

4. **What evidence exists?**
   - All patterns detected
   - Their individual scores
   - Contributing factors

5. **What reduced confidence?**
   - Conflicting alignment
   - Aged patterns
   - Unfavorable regime

## Configuration

```toml
[signal_engine]
# Quality thresholds
min_confluence_score = 70          # Minimum to emit
min_signal_quality = "A"           # A+, A, or B
min_risk_reward = 2.0              # 1:2 minimum

# Timeframes
analysis_timeframes = ["H4", "H1", "M30", "M15"]
execution_timeframe = "M15"

# Regime detection
volatility_lookback = 50
volatility_percentile_threshold = 0.75
trend_lookback = 20

# Pattern parameters
swing_pivot_bars = 3
ob_max_age_bars = 30
fvg_tolerance_percent = 0.1
min_displacement_atr_multiple = 1.5

# Confluence weights (can override defaults)
[confluence_weights]
htf_alignment = 0.20
regime = 0.15
# ... etc

# Filters
[filters]
session_filter = true
regime_filter = true
duplicate_suppression_seconds = 300
```

## Shadow Mode Operation

```rust
pub struct ShadowModeConfig {
    pub enabled: bool,
    pub emit_to_pubsub: bool,          // Publish for comparison
    pub store_comparisons: bool,        // Save to signal_comparisons table
    pub comparison_sample_rate: f64,   // 1.0 = compare all
}
```

## Performance Requirements

- **Latency:** P99 < 10ms from data to signal (cached structures)
- **Throughput:** 1000+ candles/second per symbol
- **Memory:** < 512MB for 100 symbols, all timeframes
- **CPU:** Single-core efficient, multi-core parallelizable

## Limitations

1. **Historical Replay Only:** No forward simulation capability
2. **Fixed Timeframes:** H4/H1/M30/M15 only (no custom)
3. **No News/Event Analysis:** Price-action only
4. **No Cross-Asset Correlation:** Per-symbol analysis
5. **Deterministic:** Same input always produces same output

## Dependencies

- Core: `tokio`, `tracing`, `serde`
- gRPC: `tonic`, `prost`
- Math: `rust_decimal`, `statrs`
- Protos: `apex-protos` (workspace)

---

*Document Version: 1.0*  
*Last Updated: 2026-06-14*
