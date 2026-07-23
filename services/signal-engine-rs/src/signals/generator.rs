use crate::confidence::calculator::ConfidenceCalculator;
use crate::config::Config;
use crate::confluence::engine::ConfluenceEngine;
use crate::error::{Result, SignalEngineError};
use crate::mtf::MTFAnalyzer;
use crate::regime::RegimeDetector;
use crate::signals::result::SignalDirection;
use crate::signals::{MarketContext, SignalResult};
use crate::smc::SMCEngine;
use crate::structure::StructureAnalyzer;
use rust_decimal::Decimal;
use time::OffsetDateTime;

pub struct SignalGenerator {
    structure_analyzer: StructureAnalyzer,
    smc_engine: SMCEngine,
    regime_detector: RegimeDetector,
    mtf_analyzer: MTFAnalyzer,
    confluence_engine: ConfluenceEngine,
    confidence_calculator: ConfidenceCalculator,
    config: Config,
}

impl SignalGenerator {
    pub fn new(config: &Config) -> Self {
        Self {
            structure_analyzer: StructureAnalyzer::new(config),
            smc_engine: SMCEngine::new(),
            regime_detector: RegimeDetector::new(config),
            mtf_analyzer: MTFAnalyzer::new(config),
            confluence_engine: ConfluenceEngine::new(config.min_confluence_score),
            confidence_calculator: ConfidenceCalculator::new(),
            config: config.clone(),
        }
    }

    pub fn generate(&self, context: &MarketContext) -> Result<Option<SignalResult>> {
        // 1. Core Structure
        let structure = self.structure_analyzer.analyze(&context.candles)?;

        // Find best available execution timeframe candles
        let exec_tf = std::iter::once(&self.config.execution_timeframe)
            .chain(self.config.timeframes.iter())
            .find(|tf| context.candles.contains_key(tf.as_str()))
            .map(String::as_str)
            .ok_or_else(|| SignalEngineError::MissingTimeframe { timeframe: self.config.execution_timeframe.clone() })?;

        // Extract swings from structure for SMC
        let mut tf_swings = structure.swing_highs.clone();
        tf_swings.extend(structure.swing_lows.clone());
        tf_swings.sort_by_key(|s| s.index);

        let mut swings = std::collections::HashMap::new();
        swings.insert(exec_tf.to_string(), tf_swings);

        // 2. SMC Context
        let smc_analysis = self.smc_engine.analyze(&context.candles, &swings);
        let tf_smc = match smc_analysis.get(exec_tf).or_else(|| smc_analysis.values().next()) {
            Some(s) => s,
            None => return Ok(None),
        };

        // 3. Market Regime
        let regime = self.regime_detector.detect(&context.candles)?;

        // 4. MTF Alignment
        let mtf = self.mtf_analyzer.analyze(&context.candles, &structure)?;

        // Determine direction based on MTF alignment
        let direction = match mtf.bias {
            crate::mtf::types::MarketBias::Bullish
            | crate::mtf::types::MarketBias::StrongBullish => SignalDirection::Long,
            crate::mtf::types::MarketBias::Bearish
            | crate::mtf::types::MarketBias::StrongBearish => SignalDirection::Short,
            _ => {
                tracing::info!("[SignalEval] {} MTF bias is {:?} (no directional bias)", context.symbol, mtf.bias);
                return Ok(None);
            }
        };

        // Extract Entry, Stop Loss, Take Profit
        let base_candles = context
            .candles
            .get(exec_tf)
            .or_else(|| context.candles.values().next())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let current_price = base_candles.last().map(|c| c.close).unwrap_or_default();

        let atr = crate::market_data::calculate_atr(base_candles, self.config.atr_period)
            .ok_or_else(|| SignalEngineError::Calculation(format!(
                "{} candles required for ATR({}) on {}",
                self.config.atr_period, self.config.atr_period, exec_tf
            )))?;

        let (stop_loss, take_profit) = match direction {
            SignalDirection::Long => {
                (
                    Some(current_price - atr),
                    Some(current_price + (atr * Decimal::from(2))),
                )
            }
            SignalDirection::Short => {
                (
                    Some(current_price + atr),
                    Some(current_price - (atr * Decimal::from(2))),
                )
            }
            _ => (None, None),
        };

        let risk = (current_price - stop_loss.unwrap_or(current_price)).abs();
        let reward = (take_profit.unwrap_or(current_price) - current_price).abs();
        let risk_reward = if risk.is_zero() { Decimal::ZERO } else { reward / risk };
        if risk.is_zero() || risk_reward < Decimal::try_from(self.config.min_risk_reward).map_err(|e| SignalEngineError::Internal(e.to_string()))? {
            tracing::warn!(symbol = %context.symbol, timeframe = %exec_tf, risk_reward = %risk_reward, minimum = self.config.min_risk_reward, "Signal rejected below minimum risk/reward");
            return Ok(None);
        }

        // 5. Confluence Scoring
        let confluence = self.confluence_engine.calculate_score(
            direction.clone(),
            &context.candles,
            &structure,
            &mtf,
            &regime,
            tf_smc,
            current_price,
            stop_loss.unwrap_or(Decimal::ZERO),
            take_profit.unwrap_or(Decimal::ZERO),
        );

        if confluence.total < self.config.min_confluence_score {
            tracing::info!(
                "[SignalEval] {} Direction: {:?}, Confluence score: {} < min threshold {}",
                context.symbol, direction, confluence.total, self.config.min_confluence_score
            );
            return Ok(None);
        }

        // 6. Confidence Calibration (Bayesian)
        let confidence = self.confidence_calculator.calculate(
            direction.clone(),
            &confluence,
            &structure,
            &mtf,
            &regime,
            tf_smc,
        );

        if confidence.overall < self.config.min_confidence_threshold {
            tracing::info!(
                "[SignalEval] {} Direction: {:?}, Confidence: {:.2} < min threshold {:.2}",
                context.symbol, direction, confidence.overall, self.config.min_confidence_threshold
            );
            return Ok(None);
        }

        let regime_str = match regime.regime_type {
            crate::regime::RegimeType::TrendingUp | crate::regime::RegimeType::TrendingDown => {
                "Trending".to_string()
            }
            crate::regime::RegimeType::Ranging => "Ranging".to_string(),
            crate::regime::RegimeType::HighVolatility
            | crate::regime::RegimeType::LowVolatility => "Volatile".to_string(),
            crate::regime::RegimeType::Transition => "Transition".to_string(),
            _ => "Unknown".to_string(),
        };

        let mut patterns = Vec::new();
        for bos in &tf_smc.bos_patterns {
            patterns.push(format!("BOS at {}", bos.prior_swing.price));
        }

        Ok(Some(SignalResult {
            signal_id: uuid::Uuid::new_v4().to_string(),
            symbol: context.symbol.clone(),
            timeframe: exec_tf.to_string(),
            direction,
            confidence: confidence.overall,
            confluence_score: confluence.total as f64,
            entry_price: current_price,
            stop_loss,
            take_profit,
            patterns,
            regime: regime_str,
            timestamp: OffsetDateTime::now_utc(),
        }))
    }
}
