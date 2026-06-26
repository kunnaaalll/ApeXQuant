use rust_decimal::Decimal;

#[derive(Debug, Clone, Default)]
pub struct SimulationMetrics {
    pub events_processed: u64,
    pub total_slippage: Decimal,
    pub total_spread_cost: Decimal,
    pub rejections: u64,
}
