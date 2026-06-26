use std::collections::HashSet;

pub struct IdempotencyGuard {
    processed_orders: HashSet<String>,
    processed_fills: HashSet<String>,
}

impl IdempotencyGuard {
    pub fn new() -> Self {
        Self {
            processed_orders: HashSet::new(),
            processed_fills: HashSet::new(),
        }
    }

    pub fn check_and_mark_order(&mut self, order_id: &str) -> Result<(), &'static str> {
        if self.processed_orders.contains(order_id) {
            return Err("Duplicate order execution detected");
        }
        self.processed_orders.insert(order_id.to_string());
        Ok(())
    }

    pub fn check_and_mark_fill(&mut self, fill_id: &str) -> Result<(), &'static str> {
        if self.processed_fills.contains(fill_id) {
            return Err("Duplicate fill execution detected");
        }
        self.processed_fills.insert(fill_id.to_string());
        Ok(())
    }
}

impl Default for IdempotencyGuard {
    fn default() -> Self {
        Self::new()
    }
}
