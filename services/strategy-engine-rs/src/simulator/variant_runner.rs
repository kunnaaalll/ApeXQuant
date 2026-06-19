#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantComparison {
    pub best_variant: String,
    pub worst_variant: String,
}

impl VariantComparison {
    pub fn new(best: String, worst: String) -> Self {
        Self {
            best_variant: best,
            worst_variant: worst,
        }
    }
}
