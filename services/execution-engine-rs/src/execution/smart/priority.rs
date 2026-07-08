#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}
