#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FillQualityGrade {
    Terrible,
    Poor,
    Normal,
    Good,
    Excellent,
}

impl Default for FillQualityGrade {
    fn default() -> Self {
        Self::Normal
    }
}
