use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationEvent {
    ValueAdded { amount: Decimal },
    ValueSubtracted { amount: Decimal },
    Multiplied { factor: Decimal },
}
