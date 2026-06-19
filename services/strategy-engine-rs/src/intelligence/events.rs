// Creating src/intelligence/events.rs
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntelligenceEvent {
    EdgeUpdated {
        expectancy: Decimal,
        stability: Decimal,
    },
    ConfidenceUpdated {
        new_score: Decimal,
    },
    StreakUpdated {
        win_streak: u32,
        loss_streak: u32,
    },
    DriftDetected {
        drift_amount: Decimal,
    },
    EvidenceAccumulated {
        weight: Decimal,
    },
    MemoryUpdated {
        events_count: usize,
    },
    RecommendationChanged {
        // Will match the enums we define
        from_rec: u8,
        to_rec: u8,
    },
}
