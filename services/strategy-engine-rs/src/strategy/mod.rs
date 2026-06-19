use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyProfile {
    pub strategy_id: String,
    pub name: String,
    pub description: String,
    pub creation_date: i64,
    pub version: String,
    pub regime_preferences: Vec<String>,
    pub symbol_preferences: Vec<String>,
    pub timeframe_preferences: Vec<String>,
    pub pattern_preferences: Vec<String>,
    pub edge_score: Decimal,
    pub expectancy: Decimal,
    pub confidence: Decimal,
    pub stability: Decimal,
    pub drawdown: Decimal,
    pub health_score: Decimal,
}
