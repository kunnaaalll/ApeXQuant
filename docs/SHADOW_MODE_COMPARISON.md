# Shadow Mode Comparison Guide

## Overview

Shadow Mode allows the Rust Signal Engine to run in parallel with the existing TypeScript engine without affecting live trading. Both engines process the same market data and produce signals for comparison.

## Goals

1. **Validate Rust implementation** against proven TypeScript logic
2. **Measure parity** between engines (agreement rate)
3. **Identify discrepancies** for investigation
4. **Build confidence** before live deployment

## Architecture

```
                    Market Data (MT5)
                          │
              ┌───────────┴───────────┐
              │                       │
              ▼                       ▼
    ┌─────────────────┐   ┌─────────────────┐
    │  TS Signal Eng  │   │ Rust Signal Eng │
    │   (Production)  │   │   (Shadow Mode) │
    └────────┬────────┘   └────────┬────────┘
             │                     │
             ▼                     ▼
    ┌─────────────────┐   ┌─────────────────┐
    │   Live Orders   │   │  Comparison DB  │
    └─────────────────┘   └────────┬────────┘
                                   │
                                   ▼
                          ┌─────────────────┐
                          │  Parity Reports │
                          │  & Analysis     │
                          └─────────────────┘
```

## Data Flow

### 1. Market Data Broadcast

```rust
// Event Bus publishes to both engines
event_bus.publish(Event::MarketData {
    symbol: "EURUSD",
    timeframe: Timeframe::M15,
    candles: chunk,
    shadow_mode: true,  // Both engines receive
});
```

### 2. Parallel Processing

```typescript
// TypeScript Engine (existing)
const tsSignal = await tsSignalEngine.process(data);
if (tsSignal && tsSignal.quality >= 'A') {
    await executionEngine.submit(tsSignal);  // Live orders
}
```

```rust
// Rust Engine (shadow)
let rust_signal = rust_signal_engine.process(data).await;
// Not sent to execution - stored for comparison
if let Some(signal) = rust_signal {
    comparison_store.save(
        symbol,
        data.timestamp,
        tsSignal,      // Option<Signal>
        rust_signal,   // Option<Signal>
    ).await;
}
```

### 3. Comparison Storage

```sql
-- signal_comparisons table
CREATE TABLE signal_comparisons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    comparison_run_id UUID NOT NULL,

    -- Timing
    symbol VARCHAR(10) NOT NULL,
    timeframe VARCHAR(5) NOT NULL,
    triggered_at TIMESTAMPTZ NOT NULL,
    compared_at TIMESTAMPTZ DEFAULT NOW(),

    -- TypeScript Signal (if any)
    ts_signal_id UUID,
    ts_direction VARCHAR(4),
    ts_confluence_score DECIMAL(4,1),
    ts_confidence DECIMAL(3,2),
    ts_entry_price DECIMAL(18,8),
    ts_stop_price DECIMAL(18,8),
    ts_target_price DECIMAL(18,8),
    ts_patterns JSONB,

    -- Rust Signal (if any)
    rust_signal_id UUID,
    rust_direction VARCHAR(4),
    rust_confluence_score DECIMAL(4,1),
    rust_confidence DECIMAL(3,2),
    rust_entry_price DECIMAL(18,8),
    rust_stop_price DECIMAL(18,8),
    rust_target_price DECIMAL(18,8),
    rust_patterns JSONB,

    -- Comparison Results
    agreement_type VARCHAR(20),  -- 'full', 'partial', 'direction_mismatch', 'ts_only', 'rust_only', 'none'
    direction_agreement BOOLEAN,
    confluence_diff DECIMAL(4,1),
    entry_diff_pips DECIMAL(8,4),
    stop_diff_pips DECIMAL(8,4),
    target_diff_pips DECIMAL(8,4),

    -- Analysis
    investigation_status VARCHAR(20) DEFAULT 'pending',  -- pending, investigating, resolved, expected
    investigation_notes TEXT,
    root_cause VARCHAR(50),

    -- Indexes
    INDEX idx_triggered (triggered_at),
    INDEX idx_symbol (symbol, triggered_at),
    INDEX idx_agreement (agreement_type),
    INDEX idx_investigation (investigation_status)
);
```

## Agreement Types

| Type | Description | Example |
|------|-------------|---------|
| `full` | Both engines agree on everything | Same direction, similar scores, similar levels |
| `partial` | Same direction, different details | Both bullish, different confluence scores |
| `direction_mismatch` | Opposite directions | TS says buy, Rust says sell |
| `ts_only` | Only TypeScript generated signal | Rust produced no signal |
| `rust_only` | Only Rust generated signal | TypeScript produced no signal |
| `none` | Neither generated signal | Normal - no setup present |

## Configuration

```toml
[shadow_mode]
enabled = true
emit_to_pubsub = true          # Publish for dashboard/analysis
store_comparisons = true        # Save to PostgreSQL
comparison_sample_rate = 1.0    # 1.0 = compare all, 0.1 = 10%

[shadow_mode.comparison]
confluence_tolerance = 5.0      # Score difference threshold
entry_tolerance_pips = 2.0      # Entry zone tolerance
pattern_match_threshold = 0.7   # 70% pattern overlap = match

[shadow_mode.reporting]
generate_hourly = true
generate_daily = true
alert_on_direction_mismatch = true
alert_agreement_below = 0.90    # Alert if agreement < 90%
```

## Parity Report Format

### SignalParityReport.md (Auto-Generated)

```markdown
# Signal Parity Report

**Period:** 2026-06-14 00:00 - 23:59 UTC  
**Comparison Run ID:** `018f...`  
**Generated:** 2026-06-15 00:05 UTC

## Summary

| Metric | Value |
|--------|-------|
| Total Comparisons | 1,247 |
| Agreement Rate | 94.2% |
| Direction Mismatches | 12 |
| TS-Only Signals | 38 |
| Rust-Only Signals | 29 |

## Agreement Breakdown

| Type | Count | Percentage |
|------|-------|------------|
| Full Agreement | 1,102 | 88.4% |
| Partial Agreement | 68 | 5.5% |
| Direction Mismatch | 12 | 1.0% |
| TS Only | 38 | 3.0% |
| Rust Only | 29 | 2.3% |
| None | 0 | 0.0% |

## By Symbol

| Symbol | Comparisons | Agreement Rate | Mismatches |
|--------|-------------|----------------|------------|
| EURUSD | 450 | 96.2% | 3 |
| GBPUSD | 412 | 93.0% | 5 |
| XAUUSD | 385 | 92.5% | 4 |

## Investigation Queue

### High Priority (Direction Mismatches)

| Time | Symbol | TypeScript | Rust | Investigation |
|------|--------|------------|------|---------------|
| 08:23:14 | EURUSD | BUY(score: 85) | SELL(score: 78) | [#124] OB detection difference |
| 14:15:32 | GBPUSD | SELL(score: 72) | BUY(score: 68) | [#125] Trend classification |

### Medium Priority (Threshold Differences)

[...]

## Trend Analysis

### Agreement Rate (7-day)
```
06/08: 91.2%
06/09: 92.5%
06/10: 93.1%
06/11: 93.8%
06/12: 94.0%
06/13: 94.5%
06/14: 94.2% ←
```

## Root Cause Categories

| Category | Count | Percentage |
|----------|-------|------------|
| OB Freshness Threshold | 18 | 23% |
| FVG Fill Calculation | 12 | 15% |
| Trend Structure Logic | 15 | 19% |
| MTF Weighting | 8 | 10% |
| Unknown/Investigating | 26 | 33% |

## Recommendations

1. **Address OB freshness (#18):** Rust uses 20 bars, TS uses 30 bars
2. **Review trend classification:** 5 mismatches from swing detection
3. **Consider alignment:** Agreement rate >94% - approaching threshold

---
*Report generated by parity_analyzer v1.0*
```

## Comparison Logic

```rust
pub struct SignalComparator;

impl SignalComparator {
    pub fn compare(
        ts: Option<&Signal>,
        rust: Option<&Signal>,
        config: &ComparisonConfig,
    ) -> ComparisonResult {
        match (ts, rust) {
            (None, None) => ComparisonResult {
                agreement_type: AgreementType::None,
                ..Default::default()
            },

            (Some(ts_sig), None) => ComparisonResult {
                agreement_type: AgreementType::TsOnly,
                ts_signal: Some(ts_sig.into()),
                rust_signal: None,
                ..Default::default()
            },

            (None, Some(rust_sig)) => ComparisonResult {
                agreement_type: AgreementType::RustOnly,
                ts_signal: None,
                rust_signal: Some(rust_sig.into()),
                ..Default::default()
            },

            (Some(ts_sig), Some(rust_sig)) => {
                Self::compare_both(ts_sig, rust_sig, config)
            }
        }
    }

    fn compare_both(
        ts: &Signal,
        rust: &Signal,
        config: &ComparisonConfig,
    ) -> ComparisonResult {
        let direction_agreement = ts.direction == rust.direction;

        let confluence_diff = (ts.confluence_score.total as f64 -
                               rust.confluence_score.total as f64).abs();

        let entry_diff_pips = Self::price_diff_pips(
            ts.suggested_entry,
            rust.suggested_entry,
            ts.symbol
        );

        let agreement_type = if direction_agreement
            && confluence_diff <= config.confluence_tolerance
            && entry_diff_pips <= config.entry_tolerance_pips {
            AgreementType::Full
        } else if direction_agreement {
            AgreementType::Partial
        } else {
            AgreementType::DirectionMismatch
        };

        ComparisonResult {
            agreement_type,
            direction_agreement,
            confluence_diff,
            entry_diff_pips,
            stop_diff_pips: Self::price_diff_pips(ts.stop_loss, rust.stop_loss, ts.symbol),
            target_diff_pips: Self::price_diff_pips(
                ts.suggested_take_profit,
                rust.suggested_take_profit,
                ts.symbol
            ),
            pattern_overlap: Self::calculate_pattern_overlap(
                &ts.patterns_detected,
                &rust.patterns_detected
            ),
            ts_signal: Some(ts.into()),
            rust_signal: Some(rust.into()),
        }
    }
}
```

## Alerting

```rust
pub struct ParityAlertSystem;

impl ParityAlertSystem {
    pub async fn check_alerts(&self, metrics: &ParityMetrics) {
        // Alert if agreement drops below threshold
        if metrics.agreement_rate < self.config.alert_agreement_below {
            self.send_alert(Alert::LowAgreement {
                current_rate: metrics.agreement_rate,
                threshold: self.config.alert_agreement_below,
                window: metrics.window,
            }).await;
        }

        // Alert on immediate direction mismatches
        for mismatch in &metrics.recent_direction_mismatches {
            self.send_alert(Alert::DirectionMismatch {
                symbol: mismatch.symbol,
                ts_direction: mismatch.ts_direction,
                rust_direction: mismatch.rust_direction,
                confluence_diff: mismatch.confluence_diff,
            }).await;
        }

        // Alert if Rust generating significantly more signals
        let rust_ts_ratio = metrics.rust_signal_count as f64
            / metrics.ts_signal_count.max(1) as f64;
        if rust_ts_ratio > 1.5 {
            self.send_alert(Alert::Overgeneration {
                rust_count: metrics.rust_signal_count,
                ts_count: metrics.ts_signal_count,
                ratio: rust_ts_ratio,
            }).await;
        }
    }
}
```

## Investigation Workflow

```
1. Parity report identifies discrepancy
   ↓
2. Automatic ticket creation
   ↓
3. Engineer examines specific case:
   - Market data at time
   - Both engine outputs
   - Internal calculations
   ↓
4. Root cause classification:
   - Bug in Rust (fix Rust)
   - Bug in TS (note for TS fix)
   - Expected difference (document)
   - Tuning difference (align configs)
   ↓
5. Fix implemented
   ↓
6. Re-run historical data
   ↓
7. Verify fix resolved issue
   ↓
8. Close investigation
```

## Go-Live Criteria

| Metric | Threshold | Duration |
|--------|-----------|----------|
| Agreement Rate | >95% | 7 consecutive days |
| Direction Mismatches | 0 | 3 consecutive days |
| Investigation Queue | <5 open | At go-live |
| No Critical Bugs | 0 open | 7 consecutive days |
| Performance | P99 < 10ms | 7 consecutive days |

## Transition to Live

```rust
// Gradual rollout
pub enum EngineMode {
    /// Shadow mode - compare only
    Shadow,
    /// Advisory mode - emit signals but don't trade
    Advisory,
    /// Paper mode - trade on demo accounts
    Paper,
    /// Live mode - full production
    Live,
}

// Migration path:
// Week 1-2: Shadow (100%)
// Week 3-4: Advisory (emit to dashboard)
// Week 5-6: Paper (demo trading)
// Week 7+: Live (if all criteria met)
```

## Best Practices

1. **Don't chase 100% agreement** - Small differences from floating point or timing are OK
2. **Focus on direction agreement** - This matters most for P&L
3. **Document expected differences** - Some divergence is intentional
4. **Investigate systematically** - Root cause categorization prevents thrash
5. **Test fixes historically** - Re-run affected periods after fixes
6. **Track trend over time** - Agreement should improve, not degrade

---

*Document Version: 1.0*  
*Last Updated: 2026-06-14*
