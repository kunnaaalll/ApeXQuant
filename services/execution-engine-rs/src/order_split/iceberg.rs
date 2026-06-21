use super::child_orders::ChildOrder;
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct IcebergSplitter;

impl IcebergSplitter {
    pub fn split(
        parent_id: Uuid,
        total_quantity: Decimal,
        visible_quantity: Decimal,
    ) -> Vec<ChildOrder> {
        if total_quantity <= Decimal::ZERO || visible_quantity <= Decimal::ZERO {
            return vec![];
        }

        let mut child_orders = Vec::new();
        let mut remaining = total_quantity;
        let mut seq = 0;

        while remaining > Decimal::ZERO {
            let quantity = if remaining < visible_quantity {
                remaining
            } else {
                visible_quantity
            };

            child_orders.push(ChildOrder::new(parent_id, seq, quantity));
            remaining -= quantity;
            seq += 1;
        }

        child_orders
    }
}
