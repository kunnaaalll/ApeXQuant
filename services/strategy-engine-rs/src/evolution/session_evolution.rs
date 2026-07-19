#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SessionState {
    Improving,
    #[default]
    Stable,
    Deteriorating,
    Abandoned,
}
