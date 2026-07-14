use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketFeatureSet {
    pub timestamp: u64,
    pub price_action: PriceActionFeatures,
    pub volatility: VolatilityFeatures,
    pub momentum: MomentumFeatures,
    pub liquidity: LiquidityFeatures,
    pub market_structure: MarketStructureFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceActionFeatures {
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub vwap: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityFeatures {
    pub atr_14: Decimal,
    pub bollinger_width: Decimal,
    pub historical_volatility: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumFeatures {
    pub rsi_14: Decimal,
    pub macd: Decimal,
    pub macd_signal: Decimal,
    pub macd_hist: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityFeatures {
    pub volume: Decimal,
    pub order_flow_imbalance: Decimal,
    pub bid_ask_spread: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStructureFeatures {
    pub smc_trend_direction: i8, // 1 for up, -1 for down, 0 for neutral
    pub nearest_order_block_distance: Decimal,
    pub nearest_fvg_distance: Decimal,
}

pub struct FeatureEngine;

impl FeatureEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_features(&self, raw_data: &[Decimal]) -> MarketFeatureSet {
        // Deterministic feature generation
        let close = raw_data.last().cloned().unwrap_or(Decimal::ZERO);
        let high = raw_data.iter().max().cloned().unwrap_or(close);
        let low = raw_data.iter().min().cloned().unwrap_or(close);
        
        let sum: Decimal = raw_data.iter().sum();
        let count = Decimal::new(raw_data.len() as i64, 0);
        let vwap = if count > Decimal::ZERO { sum / count } else { close };
        
        MarketFeatureSet {
            timestamp: 0, // Should be passed in
            price_action: PriceActionFeatures {
                open: raw_data.first().cloned().unwrap_or(close),
                high,
                low,
                close,
                vwap,
            },
            volatility: VolatilityFeatures {
                atr_14: (high - low) / Decimal::new(2, 0),
                bollinger_width: (high - low),
                historical_volatility: Decimal::ZERO, // Complex calc omitted for brevity, but deterministic
            },
            momentum: MomentumFeatures {
                rsi_14: Decimal::new(50, 0), // Neutral
                macd: Decimal::ZERO,
                macd_signal: Decimal::ZERO,
                macd_hist: Decimal::ZERO,
            },
            liquidity: LiquidityFeatures {
                volume: Decimal::ZERO,
                order_flow_imbalance: Decimal::ZERO,
                bid_ask_spread: Decimal::ZERO,
            },
            market_structure: MarketStructureFeatures {
                smc_trend_direction: if close > vwap { 1 } else { -1 },
                nearest_order_block_distance: Decimal::ZERO,
                nearest_fvg_distance: Decimal::ZERO,
            },
        }
    }
}
