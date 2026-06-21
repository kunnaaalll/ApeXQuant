use super::child_orders::ChildOrder;
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct VwapSplitter;

impl VwapSplitter {
    pub fn split(
        parent_id: Uuid,
        total_quantity: Decimal,
        volume_weights: &[Decimal],
    ) -> Vec<ChildOrder> {
        if volume_weights.is_empty() || total_quantity <= Decimal::ZERO {
            return vec![];
        }

        let total_weight: Decimal = volume_weights.iter().sum();
        if total_weight <= Decimal::ZERO {
            return vec![];
        }

        let mut child_orders = Vec::with_capacity(volume_weights.len());
        let mut remaining = total_quantity;

        for (i, &weight) in volume_weights.iter().enumerate() {
            let quantity = if i == volume_weights.len() - 1 {
                // Last slice takes whatever is left
                remaining
            } else {
                ((total_quantity * weight) / total_weight).trunc_with_scale(4)
            };

            child_orders.push(ChildOrder::new(parent_id, i, quantity));
            remaining -= quantity;
        }

        child_orders
    }
}
