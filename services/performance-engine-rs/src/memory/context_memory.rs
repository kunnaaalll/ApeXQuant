use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ContextMemory {
    // Maps context identifier (e.g. regime or session) to its historical performance score
    pub context_scores: HashMap<String, Decimal>,
}

impl ContextMemory {
    pub fn new() -> Self {
        Self {
            context_scores: HashMap::new(),
        }
    }

    pub fn update_context(&mut self, context: String, new_score: Decimal, alpha: Decimal) {
        let prev_score = self.context_scores.get(&context).cloned().unwrap_or(new_score);
        let one_minus_alpha = rust_decimal_macros::dec!(1.0) - alpha;
        let smoothed = (new_score * alpha) + (prev_score * one_minus_alpha);
        self.context_scores.insert(context, smoothed);
    }
}
