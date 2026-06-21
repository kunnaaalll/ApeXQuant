use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildOrderStatus {
    Pending,
    Submitted,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct ChildOrder {
    pub id: Uuid,
    pub parent_id: Uuid,
    pub sequence: usize,
    pub quantity: Decimal,
    pub status: ChildOrderStatus,
}

impl ChildOrder {
    pub fn new(parent_id: Uuid, sequence: usize, quantity: Decimal) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent_id,
            sequence,
            quantity,
            status: ChildOrderStatus::Pending,
        }
    }
}
