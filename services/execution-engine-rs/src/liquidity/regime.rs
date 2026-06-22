#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Default)]
pub enum LiquidityRegime {
    Illiquid,
    Weak,
    #[default]
    Normal,
    Healthy,
    Excellent,
}

