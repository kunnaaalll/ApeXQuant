use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationSnapshot {
    pub value: Decimal,
}
