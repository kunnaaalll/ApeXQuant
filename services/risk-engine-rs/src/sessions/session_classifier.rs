//! Session classification from timestamps

use crate::sessions::{AsiaSession, LondonSession, MarketSession, NewYorkSession};
use time::{Date, OffsetDateTime, Time, Weekday};

/// Classifies timestamps into trading sessions
pub struct SessionClassifier {
    london: LondonSession,
    ny: NewYorkSession,
    asia: AsiaSession,
}

impl SessionClassifier {
    /// Create new classifier
    pub fn new() -> Self {
        Self {
            london: LondonSession::new(),
            ny: NewYorkSession::new(),
            asia: AsiaSession::new(),
        }
    }

    /// Classify a timestamp into a market session
    pub fn classify(&self, timestamp: OffsetDateTime) -> MarketSession {
        // Check for weekend
        if self.is_weekend(timestamp) {
            return MarketSession::Weekend;
        }

        let time = timestamp.time();
        let hour = time.hour();

        // Check for London-NY overlap first (highest priority)
        if self.london.is_active(timestamp) && self.ny.is_active(timestamp) {
            return MarketSession::OverlapLondonNy;
        }

        // Check for Asian overlap
        if self.asia.is_overlap(timestamp) && hour >= 23 {
            return MarketSession::OverlapAsia;
        }

        // Check individual sessions
        if self.london.is_active(timestamp) {
            // If London is active but NY isn't, check if it's overlap start
            if hour >= 13 && hour < 14 {
                return MarketSession::OverlapLondonNy;
            }
            return MarketSession::London;
        }

        if self.ny.is_active(timestamp) {
            return MarketSession::NewYork;
        }

        if self.asia.is_active(timestamp) {
            return MarketSession::Asia;
        }

        // Low liquidity periods (between sessions)
        // 17:00-21:00 UTC = between London close and Sydney open
        if hour >= 17 && hour < 21 {
            return MarketSession::LowLiquidity;
        }

        // 07:00-08:00 = between Asia and London
        if hour == 7 {
            return MarketSession::LowLiquidity;
        }

        // 22:00-23:00 = after NY close, before Tokyo
        if hour >= 22 && hour < 23 {
            return MarketSession::LowLiquidity;
        }

        // Default to Asia for remaining times
        MarketSession::Asia
    }

    /// Check if timestamp is during weekend
    fn is_weekend(&self, timestamp: OffsetDateTime) -> bool {
        let weekday = timestamp.date().weekday();
        // Sunday trading resumes ~21:00 UTC (Sydney open)
        // Saturday trading essentially closed

        match weekday {
            Weekday::Saturday => true,
            Weekday::Sunday => {
                // After Sydney open, consider it Asia session
                let time = timestamp.time();
                time.hour() < 21
            }
            _ => false,
        }
    }

    /// Get next session transition
    pub fn next_transition(&self, timestamp: OffsetDateTime) -> (Time, &'static str) {
        let time = timestamp.time();
        let hour = time.hour();

        // Define session boundaries
        let transitions: Vec<(u8, &'static str)> = vec![
            (7, "Frankfurt open"),
            (8, "London open"),
            (13, "NY pre-market"),
            (14, "NY open / London-NY overlap"),
            (17, "London close"),
            (21, "Sydney open"),
            (22, "NY close"),
            (23, "Tokyo open"),
        ];

        for (h, desc) in transitions {
            if hour < h {
                return (Time::from_hms(h, 0, 0).unwrap(), desc);
            }
        }

        // Next day
        (Time::from_hms(7, 0, 0).unwrap(), "Frankfurt open tomorrow")
    }

    /// Get time until next session
    pub fn time_to_next_session(&self, timestamp: OffsetDateTime) -> Option<time::Duration> {
        let (next_time, _) = self.next_transition(timestamp);
        let current = timestamp.time();

        let current_min = current.hour() as i64 * 60 + current.minute() as i64;
        let next_min = next_time.hour() as i64 * 60 + next_time.minute() as i64;

        let diff_mins = if next_min >= current_min {
            next_min - current_min
        } else {
            next_min + 24 * 60 - current_min
        };

        Some(time::Duration::minutes(diff_mins))
    }

    /// Check if in high-volatility period
    pub fn is_high_volatility_period(&self, timestamp: OffsetDateTime) -> bool {
        let session = self.classify(timestamp);
        let time = timestamp.time();
        let hour = time.hour();

        // London open (08:00)
        if hour == 8 && time.minute() < 30 {
            return true;
        }

        // NY open (14:00)
        if hour == 14 && time.minute() < 30 {
            return true;
        }

        // London close (17:00)
        if hour == 17 && time.minute() < 15 {
            return true;
        }

        // Major overlaps
        matches!(
            session,
            MarketSession::OverlapLondonNy | MarketSession::OverlapAsia
        )
    }

    /// Get session duration in hours
    pub fn session_duration_hours(&self, session: &MarketSession) -> u8 {
        match session {
            MarketSession::Asia => 10,
            MarketSession::London => 9,
            MarketSession::NewYork => 8,
            MarketSession::OverlapLondonNy => 3,
            MarketSession::OverlapAsia => 8,
            MarketSession::Weekend => 48,
            MarketSession::LowLiquidity => 4,
        }
    }

    /// Estimate volatility regime based on session
    pub fn estimated_volatility_regime(&self, timestamp: OffsetDateTime) -> VolatilityRegime {
        let session = self.classify(timestamp);
        let time = timestamp.time();
        let hour = time.hour();

        // Opening hours = higher volatility
        let is_opening = (hour == 8) || (hour == 14);

        match session {
            MarketSession::OverlapLondonNy => {
                if is_opening {
                    VolatilityRegime::High
                } else {
                    VolatilityRegime::Moderate
                }
            }
            MarketSession::London | MarketSession::NewYork => {
                if is_opening {
                    VolatilityRegime::High
                } else {
                    VolatilityRegime::Normal
                }
            }
            MarketSession::Asia => VolatilityRegime::Low,
            MarketSession::Weekend => VolatilityRegime::VeryLow,
            MarketSession::LowLiquidity => VolatilityRegime::Variable,
            _ => VolatilityRegime::Normal,
        }
    }
}

impl Default for SessionClassifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Volatility regime classification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VolatilityRegime {
    VeryLow,
    Low,
    Moderate,
    Normal,
    High,
    Variable,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_timestamp(weekday: Weekday, hour: u8) -> OffsetDateTime {
        // Use a known date: 2023-11-06 was a Monday
        let date = Date::from_calendar_date(2023, time::Month::November, 4).unwrap();
        let days_to_add = match weekday {
            Weekday::Monday => 3,
            Weekday::Tuesday => 4,
            Weekday::Wednesday => 5,
            Weekday::Thursday => 6,
            Weekday::Friday => 7,
            Weekday::Saturday => 8,
            Weekday::Sunday => 9,
        };

        let target_date = date + time::Duration::days(days_to_add - 4);
        let time = Time::from_hms(hour, 0, 0).unwrap();

        // Create datetime (this is simplified - real impl would handle offsets properly)
        OffsetDateTime::new_in_offset(
            target_date,
            time,
            time::UtcOffset::UTC,
        )
    }

    #[test]
    fn test_london_classification() {
        let classifier = SessionClassifier::new();
        let london_time = make_timestamp(Weekday::Monday, 10);

        let session = classifier.classify(london_time);
        assert_eq!(session, MarketSession::London);
    }

    #[test]
    fn test_overlap_classification() {
        let classifier = SessionClassifier::new();
        let overlap_time = make_timestamp(Weekday::Monday, 15);

        let session = classifier.classify(overlap_time);
        assert_eq!(session, MarketSession::OverlapLondonNy);
    }

    #[test]
    fn test_weekend_classification() {
        let classifier = SessionClassifier::new();
        let saturday = make_timestamp(Weekday::Saturday, 12);

        let session = classifier.classify(saturday);
        assert_eq!(session, MarketSession::Weekend);
    }

    #[test]
    fn test_next_transition() {
        let classifier = SessionClassifier::new();
        let morning = make_timestamp(Weekday::Monday, 9);

        let (time, desc) = classifier.next_transition(morning);
        assert_eq!(time.hour(), 13); // NY pre-market
        assert!(desc.contains("NY"));
    }
}
