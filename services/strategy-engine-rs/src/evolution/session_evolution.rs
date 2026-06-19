#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum SessionState {
    Improving,
    #[default]
    Stable,
    Deteriorating,
    Abandoned,
}

