use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapSeverity {
    Minor,
    Moderate,
    Major,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GapType {
    MissingTicks(u64),
    TimestampJump(chrono::Duration),
    DuplicatePacket,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GapEvent {
    pub severity: GapSeverity,
    pub gap_type: GapType,
    pub timestamp: DateTime<Utc>,
}

pub struct GapDetector {
    last_timestamp: Option<DateTime<Utc>>,
}

impl Default for GapDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl GapDetector {
    pub fn new() -> Self {
        Self {
            last_timestamp: None,
        }
    }

    pub fn process_tick(&mut self, timestamp: DateTime<Utc>, sequence_missing: u64, is_duplicate: bool) -> Option<GapEvent> {
        if is_duplicate {
            return Some(GapEvent {
                severity: GapSeverity::Minor,
                gap_type: GapType::DuplicatePacket,
                timestamp,
            });
        }

        if sequence_missing > 0 {
            let severity = match sequence_missing {
                1..=5 => GapSeverity::Minor,
                6..=20 => GapSeverity::Moderate,
                21..=100 => GapSeverity::Major,
                _ => GapSeverity::Critical,
            };
            self.last_timestamp = Some(timestamp);
            return Some(GapEvent {
                severity,
                gap_type: GapType::MissingTicks(sequence_missing),
                timestamp,
            });
        }

        if let Some(last) = self.last_timestamp {
            if timestamp > last {
                let duration = timestamp.signed_duration_since(last);
                if duration.num_seconds() > 5 {
                    let severity = match duration.num_seconds() {
                        6..=15 => GapSeverity::Minor,
                        16..=60 => GapSeverity::Moderate,
                        61..=300 => GapSeverity::Major,
                        _ => GapSeverity::Critical,
                    };
                    self.last_timestamp = Some(timestamp);
                    return Some(GapEvent {
                        severity,
                        gap_type: GapType::TimestampJump(duration),
                        timestamp,
                    });
                }
            } else if timestamp < last {
                 // Out of order timestamps? 
                 let duration = last.signed_duration_since(timestamp);
                 self.last_timestamp = Some(timestamp);
                 return Some(GapEvent {
                     severity: GapSeverity::Moderate,
                     gap_type: GapType::TimestampJump(-duration),
                     timestamp,
                 });
            }
        }
        
        self.last_timestamp = Some(timestamp);
        None
    }

    pub fn reset(&mut self) {
        self.last_timestamp = None;
    }
}
