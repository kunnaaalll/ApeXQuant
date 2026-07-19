#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum WeaknessLevel {
    #[default]
    Watchlist,
    Weak,
    Danger,
    Forbidden,
}
