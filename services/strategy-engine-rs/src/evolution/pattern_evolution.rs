#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum PatternState {
    #[default]
    Maturing,
    Stable,
    Fading,
    Obsolete,
}

