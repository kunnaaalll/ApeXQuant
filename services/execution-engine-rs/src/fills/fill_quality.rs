#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum FillQualityGrade {
    Terrible,
    Poor,
    #[default]
    Normal,
    Good,
    Excellent,
}
