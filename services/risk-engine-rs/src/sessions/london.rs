//! London/European session analysis
use rust_decimal::prelude::FromPrimitive;

use rust_decimal::Decimal;
use time::{OffsetDateTime, Time};

/// London session characteristics
pub struct LondonSession {
    /// Frankfurt pre-market start
    pub frankfurt_open: Time,
    /// London open
    pub london_open: Time,
    /// London close
    pub london_close: Time,
}

impl LondonSession {
    /// Create new London session analyzer
    pub fn new() -> Self {
        Self {
            frankfurt_open: Time::from_hms(7, 0, 0).unwrap(),
            london_open: Time::from_hms(8, 0, 0).unwrap(),
            london_close: Time::from_hms(17, 0, 0).unwrap(),
        }
    }

    /// Forex fix time - high volatility
    pub fn fix_time() -> Time {
        Time::from_hms(16, 0, 0).unwrap() // 16:00 London = 4pm fix
    }

    /// Check if timestamp is in London session
    pub fn is_active(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();
        let hour = time.hour();

        // Main London: 08:00 - 17:00 UTC
        // Extended: 07:00 - 17:00 (including Frankfurt)
        hour >= 7 && hour < 17
    }

    /// Check if in Frankfurt pre-market
    pub fn is_frankfurt(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();
        time.hour() == 7
    }

    /// Most active period (overlap with US pre-market)
    pub fn is_peak(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();
        let hour = time.hour();
        // 12:00 - 14:00 UTC = morning overlap
        hour >= 12 && hour < 14
    }

    /// Check if near fix time
    pub fn is_fix_window(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();
        let hour = time.hour();
        let min = time.minute();

        // 15:50 - 16:10 fix window
        (hour == 15 && min >= 50) || hour == 16 || (hour == 17 && min < 10)
    }

    /// Expected volatility characteristics
    pub fn expected_volatility(&self, symbol: &str) -> Decimal {
        let base = &symbol[..3];
        let quote = &symbol[3..6];

        // European crosses highest volatility
        if matches!(base, "EUR" | "GBP" | "CHF") && matches!(quote, "EUR" | "GBP" | "CHF") {
            Decimal::from_f64(1.0).unwrap()
        } else if matches!(base, "EUR") || matches!(quote, "EUR") {
            Decimal::from_f64(0.9).unwrap()
        } else {
            Decimal::from_f64(0.7).unwrap()
        }
    }

    /// Liquidity score
    pub fn liquidity_score(&self) -> Decimal {
        Decimal::from_f64(0.9).unwrap()
    }

    /// Best trading period within London session
    pub fn best_period(&self) -> (Time, Time) {
        (
            Time::from_hms(8, 0, 0).unwrap(),
            Time::from_hms(11, 0, 0).unwrap(), // Initial move + volatility
        )
    }

    /// London session characteristics
    pub fn characteristics(&self) -> SessionCharacteristics {
        SessionCharacteristics {
            best_pairs: vec![
                "EURUSD", "GBPUSD", "EURGBP", "USDCHF", "EURAUD", "EURJPY", "GBPJPY",
            ],
            worst_pairs: vec!["AUDJPY", "NZDUSD", "USDCAD"],
            news_sensitivity: Decimal::from_f64(0.9).unwrap(),
            trend_following_tendency: Decimal::from_f64(0.7).unwrap(),
            fix_volatility: Decimal::from_f64(1.3).unwrap(),
        }
    }

    /// Economic news release times (major ones)
    pub fn major_news_times(&self) -> Vec<Time> {
        vec![
            Time::from_hms(7, 45, 0).unwrap(), // French data
            Time::from_hms(8, 0, 0).unwrap(),  // German/EU data
            Time::from_hms(8, 55, 0).unwrap(), // German flash
            Time::from_hms(9, 30, 0).unwrap(), // UK data
            Time::from_hms(10, 0, 0).unwrap(), // EU data
        ]
    }

    /// Check if time is near major news
    pub fn is_news_time(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();
        self.major_news_times()
            .iter()
            .any(|&news_time| Self::is_near_time(time, news_time, 15))
    }

    fn is_near_time(current: Time, target: Time, minutes: u8) -> bool {
        let current_min = current.hour() as u32 * 60 + current.minute() as u32;
        let target_min = target.hour() as u32 * 60 + target.minute() as u32;
        let diff = if current_min >= target_min {
            current_min - target_min
        } else {
            target_min - current_min
        };
        diff <= minutes as u32
    }
}

impl Default for LondonSession {
    fn default() -> Self {
        Self::new()
    }
}

/// London session characteristics
#[derive(Debug, Clone)]
pub struct SessionCharacteristics {
    /// Best performing pairs
    pub best_pairs: Vec<&'static str>,
    /// Worst performing pairs
    pub worst_pairs: Vec<&'static str>,
    /// News sensitivity
    pub news_sensitivity: Decimal,
    /// Trend following tendency
    pub trend_following_tendency: Decimal,
    /// Fix time volatility multiplier
    pub fix_volatility: Decimal,
}
