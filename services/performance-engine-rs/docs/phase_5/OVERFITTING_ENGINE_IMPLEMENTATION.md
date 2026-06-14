# OVERFITTING ENGINE IMPLEMENTATION

## Purpose

The Overfitting Engine detects when a strategy's historical performance cannot be trusted because:

1. **Sample is too small** — results may not generalise.
2. **Parameters were over-optimised** — the strategy was curve-fit to historical data.
3. **Out-of-sample (OOS) performance collapses** — the edge disappears on unseen data.
4. **Parameter density is excessive** — too many variants tested per trade.

All detection is multiplicative, never additive. Confidence is **penalised**, never inflated.

---

## Components

### `SampleBiasDetector` (`sample_bias.rs`)

| Trade Count | State          | Penalty Multiplier |
|-------------|----------------|-------------------|
| < 20        | Insufficient   | 0.10              |
| 20 – 49     | Weak           | 0.50              |
| 50 – 99     | Acceptable     | 0.80              |
| 100 – 299   | Strong         | 0.95              |
| ≥ 300       | Institutional  | 1.00              |

These thresholds align exactly with `Phase 1: SampleQuality`.

---

### `ConfidencePenaltyEngine` (`confidence_penalty.rs`)

Composes three independent signals into a single multiplicative penalty:

```
combined_penalty = sample_penalty × sensitivity_penalty × oos_penalty
```

| Signal               | Description                                                  | Source              |
|----------------------|--------------------------------------------------------------|---------------------|
| `sample_penalty`     | From `SampleBiasDetector`                                    | Trade count         |
| `sensitivity_penalty`| `1 - sensitivity_ratio` — fragile = high ratio              | Parameter sweep     |
| `oos_penalty`        | OOS/IS performance ratio                                     | Forward validation  |

**Applying the penalty:**
```rust
let adjusted_confidence = raw_confidence × combined_penalty;
// clamped to [0, 1]
```

---

### `OverfitDetector` (`overfit_detector.rs`)

Evaluates three independent overfit signals:

#### Signal 1: OOS/IS Expectancy Ratio
```
ratio = out_of_sample_expectancy / in_sample_expectancy
```
| Ratio     | Penalty | Reason                |
|-----------|---------|-----------------------|
| < 0.60    | × 0.50  | Critical OOS collapse |
| 0.60–0.80 | × 0.75  | Degraded OOS          |
| > 0.80    | × 1.00  | Acceptable            |

#### Signal 2: OOS/IS Profit Factor Ratio
| Ratio     | Penalty | Reason             |
|-----------|---------|--------------------|
| < 0.70    | × 0.60  | Critical           |
| 0.70–0.85 | × 0.85  | Degraded           |
| > 0.85    | × 1.00  | Acceptable         |

#### Signal 3: Parameter Density
```
density = parameters_tested / in_sample_trades
```
| Density  | Penalty | Reason             |
|----------|---------|--------------------|
| > 0.10   | × 0.70  | Excessive          |
| 0.05–0.10 | × 0.85 | Elevated           |
| < 0.05   | × 1.00  | Acceptable         |

#### Signal 4: OOS Trade Count
| OOS trades | Penalty | Reason           |
|------------|---------|------------------|
| < 30       | × 0.60  | Insufficient OOS |
| ≥ 30       | × 1.00  | Adequate         |

---

## OverfitState Classification

| Final Combined Penalty | State       |
|------------------------|-------------|
| ≥ 0.90                 | Healthy     |
| 0.75 – 0.89            | Caution     |
| 0.60 – 0.74            | Warning     |
| 0.40 – 0.59            | Overfit     |
| < 0.40                 | Dangerous   |

---

## Invariants

1. `confidence_penalty` ∈ [0, 1] always — clamped before return.
2. All penalties are **multiplicative** — partial signals cannot inflate confidence.
3. OOS trades < 30 triggers a penalty — there is no bypass.
4. Penalty composition is **commutative and associative** — order does not matter.
5. Verified: penalty never exceeds 1.0 over 100 parameterised test cases.
