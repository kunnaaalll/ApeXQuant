#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiquidityRegime {
    Illiquid,
    Weak,
    Normal,
    Healthy,
    Excellent,
}

impl Default for LiquidityRegime {
    fn default() -> Self {
        Self::Normal
    }
}
