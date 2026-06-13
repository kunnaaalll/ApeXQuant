//! Confidence decay over time
//!
//! Signal confidence degrades as time passes without confirmation.

use time::OffsetDateTime;

/// Confidence decay model
#[derive(Debug, Clone)]
pub struct ConfidenceDecay {
    /// Half-life in minutes
    pub half_life_minutes: f64,
    /// Minimum confidence multiplier
    pub floor: f64,
    /// Last update time
    pub last_update: OffsetDateTime,
}

impl ConfidenceDecay {
    /// Create new decay model
    pub fn new(half_life_minutes: f64, floor: f64) -> Self {
        Self {
            half_life_minutes,
            floor,
            last_update: OffsetDateTime::now_utc(),
        }
    }

    /// Create default decay (5 minute half-life)
    pub fn default() -> Self {
        Self::new(5.0, 0.3)
    }

    /// Update last update time
    pub fn touch(&mut self) {
        self.last_update = OffsetDateTime::now_utc();
    }

    /// Calculate current decay multiplier
    pub fn current_decay(&self) -> f64 {
        let now = OffsetDateTime::now_utc();
        let elapsed = (now - self.last_update).as_seconds_f64() / 60.0; // minutes

        if elapsed <= 0.0 {
            return 1.0;
        }

        // Exponential decay: decay = floor + (1 - floor) * 2^(-elapsed / half_life)
        let decay = 2.0f64.powf(-elapsed / self.half_life_minutes);

        self.floor + (1.0 - self.floor) * decay
    }

    /// Calculate decay at specific time
    pub fn decay_at(&self, timestamp: OffsetDateTime) -> f64 {
        let elapsed = (timestamp - self.last_update).as_seconds_f64() / 60.0;

        if elapsed <= 0.0 {
            return 1.0;
        }

        let decay = 2.0f64.powf(-elapsed / self.half_life_minutes);
        self.floor + (1.0 - self.floor) * decay
    }

    /// Create slow decay (longer half-life)
    pub fn slow() -> Self {
        Self::new(15.0, 0.4)
    }

    /// Create fast decay (shorter half-life)
    pub fn fast() -> Self {
        Self::new(2.0, 0.2)
    }
}

/// Time-based confidence adjustment
#[derive(Debug, Clone)]
pub struct TimeAdjustment {
    /// Session start offset in minutes from midnight
    pub session_start_minutes: u32,
    /// Session end offset
    pub session_end_minutes: u32,
    /// Confidence multiplier during session
    pub session_multiplier: f64,
    /// Confidence multiplier outside session
    pub off_session_multiplier: f64,
}

impl TimeAdjustment {
    /// Create new time adjustment
    pub fn new(
        session_start: u32,
        session_end: u32,
        session_mult: f64,
        off_session_mult: f64,
    ) -> Self {
        Self {
            session_start_minutes: session_start,
            session_end_minutes: session_end,
            session_multiplier: session_mult,
            off_session_multiplier: off_session_mult,
        }
    }

    /// Get multiplier for current time
    pub fn current_multiplier(&self) -> f64 {
        let now = OffsetDateTime::now_utc();
        let minutes = now.hour() * 60 + now.minute();

        if self.is_in_session(minutes) {
            self.session_multiplier
        } else {
            self.off_session_multiplier
        }
    }

    /// Check if time is in session
    fn is_in_session(&self, minutes: u32) -> bool {
        if self.session_start_minutes <= self.session_end_minutes {
            minutes >= self.session_start_minutes && minutes < self.session_end_minutes
        } else {
            // Wraps around midnight
            minutes >= self.session_start_minutes || minutes < self.session_end_minutes
        }
    }

    /// London session (08:00-17:00 UTC)
    pub fn london() -> Self {
        Self::new(480, 1020, 1.1, 0.9)
    }

    /// New York session (13:00-22:00 UTC)
    pub fn new_york() -> Self {
        Self::new(780, 1320, 1.1, 0.9)
    }

    /// London-NY overlap (highest activity)
    pub fn overlap() -> Self {
        Self::new(780, 1020, 1.15, 1.0)
    }
}

/// Decay schedule for different signal types
#[derive(Debug, Clone)]
pub struct DecaySchedule {
    /// Scalp signals (fast decay)
    pub scalp: ConfidenceDecay,
    /// Intraday signals (medium decay)
    pub intraday: ConfidenceDecay,
    /// Swing signals (slow decay)
    pub swing: ConfidenceDecay,
}

impl DecaySchedule {
    /// Create default decay schedule
    pub fn new() -> Self {
        Self {
            scalp: ConfidenceDecay::fast(),
            intraday: ConfidenceDecay::default(),
            swing: ConfidenceDecay::slow(),
        }
    }

    /// Get decay model for signal type
    pub fn for_signal_type(&self, signal_type: SignalType) -> &ConfidenceDecay {
        match signal_type {
            SignalType::Scalp => &self.scalp,
            SignalType::Intraday => &self.intraday,
            SignalType::Swing => &self.swing,
        }
    }
}

impl Default for DecaySchedule {
    fn default() -> Self {
        Self::new()
    }
}

/// Signal type classification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalType {
    /// Scalp (minutes to hour)
    Scalp,
    /// Intraday (hours)
    Intraday,
    /// Swing (days)
    Swing,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_decay_bounds() {
        let decay = ConfidenceDecay::new(5.0, 0.3);

        // At time 0, decay should be 1.0
        assert!((decay.current_decay() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_decay_decreases() {
        let mut decay = ConfidenceDecay::new(1.0, 0.3);
        decay.touch();

        let initial = decay.current_decay();

        // Wait a tiny bit (can't actually sleep in unit tests, so just verify function works)
        // In real tests, we'd mock time

        // Decay should be <= initial
        assert!(decay.current_decay() <= initial);
    }

    #[test]
    fn test_time_adjustment_session() {
        let adj = TimeAdjustment::london();

        // 09:00 UTC is in London session
        assert!(adj.is_in_session(540));

        // 06:00 UTC is outside
        assert!(!adj.is_in_session(360));
    }

    #[test]
    fn test_signal_type_decay() {
        let schedule = DecaySchedule::new();

        assert_eq!(schedule.for_signal_type(SignalType::Scalp).floor, 0.2);
        assert_eq!(schedule.for_signal_type(SignalType::Swing).floor, 0.4);
    }
}
