// Symbol domain

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolProfile {
    pub symbol: String,
    pub pip_size: rust_decimal::Decimal,
    pub lot_size: rust_decimal::Decimal,
    pub min_volume: rust_decimal::Decimal,
    pub max_volume: rust_decimal::Decimal,
    pub price_precision: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolClass {
    Forex,
    Crypto,
    Index,
    Commodity,
    Metal,
}
