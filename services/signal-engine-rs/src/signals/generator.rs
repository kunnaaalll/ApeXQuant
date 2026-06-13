//! Signal generation orchestration

use crate::config::Config;
use crate::config::SignalQuality;
use crate::confidence::ConfidenceCalculator;
use crate::confluence::engine::{ConfluenceEngine, SignalDirection};
use crate::evidence::EvidenceCollector;
use crate::market_data::Candle;
use crate::signals::result::{DetectedPattern, SignalEvidence};
use crate::signals::{MarketContext, SignalResult, SignalValidator};
use crate::smc::SMCEngine;
use crate::structure::swings::{detect_swings, Swings};
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Main signal generator
#[derive(Debug)]
pub struct SignalGenerator {
    config: Config,
    validator: SignalValidator,
    confluence_engine: ConfluenceEngine,
    confidence_calculator: ConfidenceCalculator,
    smc_engine: SMCEngine,
}

impl SignalGenerator {
    /// Create new signal generator
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            validator: SignalValidator::new(config),
            confluence_engine: ConfluenceEngine::new(config.min_confluence_score),
            confidence_calculator: ConfidenceCalculator::new(),
            smc_engine: SMCEngine::new(),
        }
    }

    /// Generate signals from market context
    pub async fn generate(&self, context: &MarketContext) -> crate::Result<Vec<SignalResult>> {
        let mut signals = Vec::new();

        // Validate context
        if !self.is_valid_context(context) {
            debug!("Invalid market context for {} - skipping generation", context.symbol);
            return Ok(signals);
        }

        // Detect swings across all timeframes
        let swings = self.detect_all_swings(&context.candles);

        // Run SMC analysis
        let smc_analysis = self.smc_engine.analyze(&context.candles, &swings);

        // Get execution timeframe analysis
        let execution_tf = &self.config.execution_timeframe;
        let execution_candles = context.candles.get(execution_tf).cloned().unwrap_or_default();

        if execution_candles.is_empty() {
            return Ok(signals);
        }

        let execution_smc = smc_analysis.get(execution_tf).cloned().unwrap_or_default();
        let current_price = execution_candles[execution_candles.len() - 1].close;

        // Generate long signal if conditions favorable
        let long_signal = self.generate_directional_signal(
            context,
            &execution_candles,
            &execution_smc,
            SignalDirection::Long,
            current_price,
        );

        if let Some(signal) = long_signal {
            let validation = self.validator.validate(&signal);
            if validation.valid {
                signals.push(signal);
            }
        }

        // Generate short signal if conditions favorable
        let short_signal = self.generate_directional_signal(
            context,
            &execution_candles,
            &execution_smc,
            SignalDirection::Short,
            current_price,
        );

        if let Some(signal) = short_signal {
            let validation = self.validator.validate(&signal);
            if validation.valid {
                signals.push(signal);
            }
        }

        info!(
            "Generated {} signals for {} (context: {:?})",
            signals.len(),
            context.symbol,
            context.regime.regime_type
        );

        Ok(signals)
    }

    /// Generate signal for specific direction
    fn generate_directional_signal(
        &self,
        context: &MarketContext,
        candles: &[Candle],
        smc: &crate::smc::SMCAnalysis,
        direction: SignalDirection,
        current_price: Decimal,
    ) -> Option<SignalResult> {
        // Check directional prerequisites
        match direction {
            SignalDirection::Long => {
                if !context.mtf_alignment.bias.allows_long() {
                    return None;
                }
                if !smc.is_bullish_structure() && !smc.imbalance.dominant_bias.allows_long() {
                    return None;
                }
            }
            SignalDirection::Short => {
                if !context.mtf_alignment.bias.allows_short() {
                    return None;
                }
                if !smc.is_bearish_structure() && !smc.imbalance.dominant_bias.allows_short() {
                    return None;
                }
            }
        }

        // Calculate entry zone
        let entry_zone = self.calculate_entry_zone(direction, smc, current_price);
        if entry_zone.0 == Decimal::ZERO || entry_zone.1 == Decimal::ZERO {
            return None;
        }

        // Calculate stop and target
        let (stop, target, rr) = self.calculate_stop_target(
            direction,
            entry_zone,
            candles,
            smc,
            self.config.min_risk_reward,
        );

        // Calculate confluence score
        let confluence = self.confluence_engine.calculate_score(
            direction,
            &context.candles,
            &context.structure,
            &context.mtf_alignment,
            &context.regime,
            smc,
            (entry_zone.0 + entry_zone.1) / Decimal::from(2),
            stop,
            target,
        );

        // Check minimum score
        if confluence.total < self.config.min_confluence_score {
            debug!(
                "Confluence score {} below minimum {} for {} {:?}",
                confluence.total, self.config.min_confluence_score, context.symbol, direction
            );
            return None;
        }

        // Calculate confidence
        let confidence = self.confidence_calculator.calculate(
            direction, // Convert
            &confluence,
            &context.structure,
            &context.mtf_alignment,
            &context.regime,
            smc,
        );

        // Build evidence
        let mut collector = EvidenceCollector::new();
        for factor in &confluence.factors {
            collector.add_confluence_factor(factor);
        }
        let evidence_collection = collector.finalize();

        // Create signal
        let mut signal = SignalResult {
            signal_id: Uuid::new_v4(),
            symbol: context.symbol.clone(),
            direction: match direction {
                SignalDirection::Long => crate::signals::SignalDirection::Long,
                SignalDirection::Short => crate::signals::SignalDirection::Short,
            },
            confidence: confidence.overall,
            confluence_score: confluence.total,
            quality: confluence.as_grade(),
            market_regime: context.regime.regime_type,
            timeframe_alignment: context.mtf_alignment.bias,
            entry_zone_top: entry_zone.1,
            entry_zone_bottom: entry_zone.0,
            stop_zone: stop,
            target_zone: target,
            risk_reward: rr,
            patterns_detected: self.extract_patterns(smc),
            evidence: evidence_collection.factors.iter().map(|f| SignalEvidence {
                evidence_type: f.factor_type.clone(),
                description: f.interpretation.clone(),
                data: serde_json::json!({
                    "raw_value": f.raw_value,
                    "contribution": f.contribution,
                }),
            }).collect(),
            reasons: evidence_collection.reasons.clone(),
            timestamp: time::OffsetDateTime::now_utc(),
        };

        // Generate human-readable reasons
        signal.reasons.push(collector.generate_why_buy());
        signal.reasons.push(collector.generate_why_now());
        signal.reasons.push(collector.generate_why_wait());

        Some(signal)
    }

    /// Check if market context is valid for signal generation
    fn is_valid_context(&self, context: &MarketContext) -> bool {
        let execution_tf = self.config.execution_timeframe.clone();
        let has_execution_data = context.candles.contains_key(&execution_tf)
            && !context.candles.get(&execution_tf).map(|c| c.is_empty()).unwrap_or(true);

        if !has_execution_data {
            return false;
        }

        if context.regime.confidence < 0.3 {
            return false;
        }

        true
    }

    /// Detect swings across all timeframes
    fn detect_all_swings(&self, candles: &HashMap<String, Vec<Candle>>) -> HashMap<String, Vec<crate::structure::swings::SwingPoint>> {
        let mut swings = HashMap::new();

        for (tf, tf_candles) in candles {
            let tf_swings = detect_swings(tf_candles, self.config.swing_pivot_bars);
            let mut all: Vec<_> = tf_swings.highs;
            all.extend(tf_swings.lows);
            all.sort_by_key(|s| s.index);
            swings.insert(tf.clone(), all);
        }

        swings
    }

    /// Calculate entry zone based on direction and patterns
    fn calculate_entry_zone(
        &self,
        direction: SignalDirection,
        smc: &crate::smc::SMCAnalysis,
        current_price: Decimal,
    ) -> (Decimal, Decimal) {
        use crate::smc::order_blocks::OBDirection;
        use crate::smc::fvg::FVGDirection;

        let mut bottom = current_price;
        let mut top = current_price;

        // Use Order Block if available
        match direction {
            SignalDirection::Long => {
                if let Some(ob) = smc.freshest_bullish_ob() {
                    let zone = crate::smc::order_blocks::get_entry_zone(ob);
                    bottom = zone.0;
                    top = zone.1;
                } else if let Some(fvg) = smc.fvgs.iter()
                    .find(|f| matches!(f.direction, FVGDirection::Bullish) && !f.filled)
                {
                    bottom = fvg.bottom;
                    top = fvg.top;
                }
            }
            SignalDirection::Short => {
                if let Some(ob) = smc.freshest_bearish_ob() {
                    let zone = crate::smc::order_blocks::get_entry_zone(ob);
                    bottom = zone.0;
                    top = zone.1;
                } else if let Some(fvg) = smc.fvgs.iter()
                    .find(|f| matches!(f.direction, FVGDirection::Bearish) && !f.filled)
                {
                    bottom = fvg.bottom;
                    top = fvg.top;
                }
            }
        }

        // Default to current price with buffer if no patterns found
        if bottom == top {
            let buffer = current_price * Decimal::from_f64_retain(0.001).unwrap_or_default();
            bottom = current_price - buffer;
            top = current_price + buffer;
        }

        (bottom.min(top), top.max(bottom))
    }

    /// Calculate stop loss and take profit
    fn calculate_stop_target(
        &self,
        direction: SignalDirection,
        entry_zone: (Decimal, Decimal),
        candles: &[Candle],
        smc: &crate::smc::SMCAnalysis,
        min_rr: f64,
    ) -> (Decimal, Decimal, f64) {
        let entry = (entry_zone.0 + entry_zone.1) / Decimal::from(2);

        // Calculate ATR for stop sizing
        let atr = self.calculate_atr(candles);
        let atr_stop = match direction {
            SignalDirection::Long => entry - atr * Decimal::from(2),
            SignalDirection::Short => entry + atr * Decimal::from(2),
        };

        // Alternative: swing-based stop
        let swing_stop = match direction {
            SignalDirection::Long => {
                smc.order_blocks.iter()
                    .filter(|ob| matches!(ob.direction, crate::smc::order_blocks::OBDirection::Bullish))
                    .map(|ob| ob.bottom - atr * Decimal::from_f64_retain(0.5).unwrap_or_default())
                    .min()
                    .unwrap_or(atr_stop)
            }
            SignalDirection::Short => {
                smc.order_blocks.iter()
                    .filter(|ob| matches!(ob.direction, crate::smc::order_blocks::OBDirection::Bearish))
                    .map(|ob| ob.top + atr * Decimal::from_f64_retain(0.5).unwrap_or_default())
                    .max()
                    .unwrap_or(atr_stop)
            }
        };

        // Use tighter of the two stops
        let stop = match direction {
            SignalDirection::Long => swing_stop.max(atr_stop),
            SignalDirection::Short => swing_stop.min(atr_stop),
        };

        // Calculate risk
        let risk = (entry - stop).abs();
        let rr_mult = self.choose_rr_multiplier(min_rr, smc);
        let reward = risk * Decimal::from_f64_retain(rr_mult).unwrap_or_default();

        let target = match direction {
            SignalDirection::Long => entry + reward,
            SignalDirection::Short => entry - reward,
        };

        let rr_calculated = if risk == Decimal::ZERO {
            0.0
        } else {
            (reward / risk).to_f64().unwrap_or(0.0)
        };

        (stop, target, rr_calculated)
    }

    /// Choose RR multiplier based on market conditions
    fn choose_rr_multiplier(&self, min_rr: f64, smc: &crate::smc::SMCAnalysis) -> f64 {
        // Base target RR
        let base_rr = if smc.displacements.iter().any(|d| d.atr_multiple > 2.0) {
            3.0 // Strong displacement = extend target
        } else {
            2.0
        };

        base_rr.max(min_rr)
    }

    /// Calculate ATR from candles
    fn calculate_atr(&self, candles: &[Candle]) -> Decimal {
        if candles.len() < 14 {
            let sum: Decimal = candles.iter().map(|c| c.high - c.low).sum();
            return sum / Decimal::from(candles.len().max(1) as i64);
        }

        let period = 14usize;
        let sum: Decimal = candles[candles.len() - period..].iter()
            .map(|c| c.high - c.low)
            .sum();

        sum / Decimal::from(period as i64)
    }

    /// Extract detected patterns for signal
    fn extract_patterns(&self, smc: &crate::smc::SMCAnalysis) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();

        // Add BOS patterns
        for bos in &smc.bos_patterns {
            patterns.push(DetectedPattern {
                pattern_type: format!("BOS {:?}", bos.direction),
                timeframe: bos.timeframe.clone(),
                strength: bos.strength,
                location: Some(bos.level),
                confidence: bos.strength,
                metadata: serde_json::json!({"age_bars": 0}),
            });
        }

        // Add order blocks
        for ob in &smc.order_blocks {
            if !ob.mitigated && ob.age_bars < 30 {
                patterns.push(DetectedPattern {
                    pattern_type: format!("OrderBlock {:?}", ob.direction),
                    timeframe: ob.timeframe.clone(),
                    strength: ob.strength,
                    location: Some(ob.top),
                    confidence: ob.strength,
                    metadata: serde_json::json!({
                        "age_bars": ob.age_bars,
                        "mitigated": ob.mitigated,
                    }),
                });
            }
        }

        // Add FVGs
        for fvg in &smc.fvgs {
            if !fvg.filled && fvg.age_bars < 30 {
                patterns.push(DetectedPattern {
                    pattern_type: format!("FVG {:?}", fvg.direction),
                    timeframe: fvg.timeframe.clone(),
                    strength: fvg.strength,
                    location: Some(fvg.top),
                    confidence: fvg.strength,
                    metadata: serde_json::json!({
                        "age_bars": fvg.age_bars,
                        "filled": fvg.filled,
                    }),
                });
            }
        }

        patterns
    }
}
