use crate::latency::LatencyGrade;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedHealthGrade {
    Excellent,
    Good,
    Warning,
    Critical,
    Dead,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthMetrics {
    pub latency_grade: LatencyGrade,
    pub missing_ticks: u64,
    pub sequence_gaps: u64,
    pub stale_timestamps_count: u64,
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthMetrics {
    pub fn new() -> Self {
        Self {
            latency_grade: LatencyGrade::Excellent,
            missing_ticks: 0,
            sequence_gaps: 0,
            stale_timestamps_count: 0,
        }
    }

    pub fn evaluate(&self) -> FeedHealthGrade {
        if self.missing_ticks > 100 || self.stale_timestamps_count > 50 || self.latency_grade == LatencyGrade::Critical {
            return FeedHealthGrade::Dead;
        }

        if self.missing_ticks > 20 || self.sequence_gaps > 10 || self.latency_grade == LatencyGrade::Slow {
            return FeedHealthGrade::Critical;
        }

        if self.missing_ticks > 5 || self.sequence_gaps > 2 || self.latency_grade == LatencyGrade::Normal {
            return FeedHealthGrade::Warning;
        }

        if self.missing_ticks > 0 || self.sequence_gaps > 0 {
            return FeedHealthGrade::Good;
        }

        FeedHealthGrade::Excellent
    }
}
