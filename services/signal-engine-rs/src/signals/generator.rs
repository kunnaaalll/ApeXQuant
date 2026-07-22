use crate::confidence::calculator::ConfidenceCalculator;
use crate::config::Config;
use crate::confluence::engine::ConfluenceEngine;
use crate::error::Result;
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

        // Extract swings from structure for SMC
        let mut m15_swings = structure.swing_highs.clone();
        m15_swings.extend(structure.swing_lows.clone());
        m15_swings.sort_by_key(|s| s.index);

        let mut swings = std::collections::HashMap::new();
        swings.insert("M15".to_string(), m15_swings);

        // 2. SMC Context
        let smc_analysis = self.smc_engine.analyze(&context.candles, &swings);
        let tf_smc = match smc_analysis.get("M15") {
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
            _ => return Ok(None), // No clear direction despite score
        };

        // Extract Entry, Stop Loss, Take Profit
        let base_candles = context
            .candles
            .get("M15")
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let current_price = base_candles.last().map(|c| c.close).unwrap_or_default();

        // Basic ATR or structure based stops (placeholder simple calc for now)
        let atr_approx = Decimal::from_str_exact("0.0050").unwrap_or_default(); // Example placeholder

        let (stop_loss, take_profit) = match direction {
            SignalDirection::Long => {
                (
                    Some(current_price - atr_approx),
                    Some(current_price + (atr_approx * Decimal::from(2))), // 2R target
                )
            }
            SignalDirection::Short => {
                (
                    Some(current_price + atr_approx),
                    Some(current_price - (atr_approx * Decimal::from(2))), // 2R target
                )
            }
            _ => (None, None),
        };

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
