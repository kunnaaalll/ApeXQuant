use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureWindow {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    FourHours,
    OneDay,
}

impl FeatureWindow {
    pub fn to_seconds(&self) -> u64 {
        match self {
            FeatureWindow::OneMinute => 60,
            FeatureWindow::FiveMinutes => 300,
            FeatureWindow::FifteenMinutes => 900,
            FeatureWindow::OneHour => 3600,
            FeatureWindow::FourHours => 14400,
            FeatureWindow::OneDay => 86400,
        }
    }
}
