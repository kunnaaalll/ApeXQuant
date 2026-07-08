#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Urgency {
    Patient,
    #[default]
    Balanced,
    Aggressive,
    Emergency,
}
