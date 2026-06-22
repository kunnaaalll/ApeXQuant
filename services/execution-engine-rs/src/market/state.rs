#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketState {
    Healthy,
    Normal,
    Stressed,
    Dislocated,
    Closed,
}
