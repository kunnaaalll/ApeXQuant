use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketClass {
    Forex,
    Indices,
    Metals,
    Crypto,
    Futures,
}

#[derive(Debug, Clone)]
pub struct CrossMarketResult {
    pub strategy_id: Uuid,
    pub market: MarketClass,
    pub score: Decimal,
    pub overfit_penalty: Decimal,
}

pub trait CrossMarketValidator {
    fn validate(&self, strategy_id: Uuid, target_markets: &[MarketClass]) -> Vec<CrossMarketResult>;
}

pub struct StandardValidator;

impl CrossMarketValidator for StandardValidator {
    fn validate(&self, strategy_id: Uuid, target_markets: &[MarketClass]) -> Vec<CrossMarketResult> {
        let mut results = Vec::new();
        for &market in target_markets {
            results.push(CrossMarketResult {
                strategy_id,
                market,
                score: Decimal::ZERO,
                overfit_penalty: Decimal::ZERO,
            });
        }
        results
    }
}
