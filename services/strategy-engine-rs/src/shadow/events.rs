use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShadowEvent {
    ComparisonProcessed { total_difference: Decimal },
    DemotionTriggered,
    StateTransitioned { old_state: u8, new_state: u8 },
}
