#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketRegime {
    Trending,
    Range,
    Expansion,
    Contraction,
    Transition,
}
