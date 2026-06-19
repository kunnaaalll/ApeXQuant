#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Default)]
pub enum WeaknessLevel {
    #[default]
    Watchlist,
    Weak,
    Danger,
    Forbidden,
}

