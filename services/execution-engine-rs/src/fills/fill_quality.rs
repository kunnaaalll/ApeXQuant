#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Default)]
pub enum FillQualityGrade {
    Terrible,
    Poor,
    #[default]
    Normal,
    Good,
    Excellent,
}

