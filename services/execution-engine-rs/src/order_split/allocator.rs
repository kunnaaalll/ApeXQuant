//! Allocator module for distributing filled quantities across child orders and parents
use rust_decimal::Decimal;

pub struct FillAllocator;

impl FillAllocator {
    pub fn allocate_to_parent(
        child_fill_qty: Decimal,
        child_fill_price: Decimal,
    ) -> (Decimal, Decimal) {
        // In a real implementation this would distribute back to the parent order
        (child_fill_qty, child_fill_price)
    }
}
