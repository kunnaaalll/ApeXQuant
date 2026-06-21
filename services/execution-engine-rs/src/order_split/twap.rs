use super::child_orders::ChildOrder;
use rust_decimal::Decimal;
use uuid::Uuid;
use rust_decimal::prelude::FromPrimitive;

pub struct TwapSplitter;

impl TwapSplitter {
    pub fn split(
        parent_id: Uuid,
        total_quantity: Decimal,
        slices: usize,
    ) -> Vec<ChildOrder> {
        if slices == 0 || total_quantity <= Decimal::ZERO {
            return vec![];
        }

        let slices_dec = Decimal::from_usize(slices).unwrap_or(Decimal::ONE);
        let base_slice = (total_quantity / slices_dec).trunc_with_scale(4);
        
        let mut child_orders = Vec::with_capacity(slices);
        let mut remaining = total_quantity;

        for i in 0..slices {
            let quantity = if i == slices - 1 {
                // Last slice takes whatever is left to avoid rounding losses
                remaining
            } else {
                base_slice
            };

            child_orders.push(ChildOrder::new(parent_id, i, quantity));
            remaining -= quantity;
        }

        child_orders
    }
}
