use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct FocusRecommendation {
    pub best_symbols: Vec<String>,
    pub best_patterns: Vec<String>,
    pub best_regimes: Vec<String>,
    pub best_sessions: Vec<String>,
    pub best_timeframes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FocusEngine {
    pub max_items: usize,
}

impl FocusEngine {
    pub fn new(max_items: usize) -> Self {
        Self { max_items }
    }

    pub fn generate_focus(&self, mut ranked_symbols: Vec<(String, Decimal)>) -> Vec<String> {
        ranked_symbols.sort_by(|a, b| b.1.cmp(&a.1)); // Descending order
        ranked_symbols.into_iter()
            .take(self.max_items)
            .map(|(name, _score)| name)
            .collect()
    }
}
