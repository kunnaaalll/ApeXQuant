//! Asian session analysis
use rust_decimal::prelude::FromPrimitive;

use rust_decimal::Decimal;
use time::{OffsetDateTime, Time};

/// Asian session characteristics
pub struct AsiaSession;

impl AsiaSession {
    /// Create new Asian session analyzer
    pub fn new() -> Self {
        Self
    }

    /// Standard Sydney open (UTC)
    pub fn sydney_open() -> Time {
        Time::from_hms(21, 0, 0).unwrap() // 21:00 UTC = 08:00 Sydney
    }

    /// Standard Tokyo open (UTC)
    pub fn tokyo_open() -> Time {
        Time::from_hms(23, 0, 0).unwrap() // 23:00 UTC = 08:00 Tokyo
    }

    /// Asian session close (UTC)
    pub fn session_close() -> Time {
        Time::from_hms(7, 0, 0).unwrap() // Tokyo close
    }

    /// Check if timestamp is in Asian session
    pub fn is_active(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();
        let hour = time.hour();

        // Asian session: 21:00 UTC - 07:00 UTC
        hour >= 21 || hour < 7
    }

    /// Asian session typically has lower volatility for majors
    pub fn expected_volatility(&self, symbol: &str) -> Decimal {
        let base = &symbol[..3];

        match base {
            "JPY" | "AUD" | "NZD" => Decimal::from_f64(0.8).unwrap(), // Lower vol for Asian pairs
            _ => Decimal::from_f64(0.6).unwrap(),                      // Higher vol reduction for others
        }
    }

    /// Get liquidity score for Asian session
    pub fn liquidity_score(&self) -> Decimal {
        Decimal::from_f64(0.5).unwrap()
    }

    /// Asian session specific characteristics
    pub fn characteristics(&self) -> SessionCharacteristics {
        SessionCharacteristics {
            best_pairs: vec!["AUDJPY", "NZDJPY", "EURJPY", "USDJPY", "AUDNZD"],
            worst_pairs: vec!["EURUSD", "GBPUSD", "EURGBP"],
            news_sensitivity: Decimal::from_f64(0.7).unwrap(),
            mean_reversion_tendency: Decimal::from_f64(0.6).unwrap(),
        }
    }

    /// Check if in Sydney-Tokyo overlap
    pub fn is_overlap(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();
        let hour = time.hour();

        // Overlap: 23:00 - 07:00 UTC
        hour >= 23 || hour < 7
    }
}

impl Default for AsiaSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Characteristics of Asian session trading
#[derive(Debug, Clone)]
pub struct SessionCharacteristics {
    /// Best performing pairs
    pub best_pairs: Vec<&'static str>,
    /// Worst performing pairs
    pub worst_pairs: Vec<&'static str>,
    /// Sensitivity to news events
    pub news_sensitivity: Decimal,
    /// Tendency toward mean reversion
    pub mean_reversion_tendency: Decimal,
}
