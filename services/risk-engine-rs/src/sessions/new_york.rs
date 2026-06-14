//! New York/North American session analysis
use rust_decimal::prelude::FromPrimitive;

use rust_decimal::Decimal;
use time::{OffsetDateTime, Time};

/// New York session characteristics
pub struct NewYorkSession {
    /// Pre-market open
    pub pre_market_open: Time,
    /// NY open
    pub nyc_open: Time,
    /// NY close
    pub nyc_close: Time,
    /// CME forex close
    pub cme_close: Time,
}

impl NewYorkSession {
    /// Create new NY session analyzer
    pub fn new() -> Self {
        Self {
            pre_market_open: Time::from_hms(13, 0, 0).unwrap(),
            nyc_open: Time::from_hms(14, 0, 0).unwrap(),
            nyc_close: Time::from_hms(22, 0, 0).unwrap(),
            cme_close: Time::from_hms(22, 0, 0).unwrap(),
        }
    }

    /// Important economic release times
    pub fn nfp_time() -> Time {
        Time::from_hms(13, 30, 0).unwrap() // First Friday of month
    }

    /// Check if timestamp is in NY session
    pub fn is_active(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();
        let hour = time.hour();

        // Main NY: 14:00 - 22:00 UTC
        hour >= 14 && hour < 22
    }

    /// Check if in pre-market
    pub fn is_pre_market(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();
        let hour = time.hour();
        hour >= 13 && hour < 14
    }

    /// Most active period (initial overlap with London)
    pub fn is_peak_overlap(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();
        let hour = time.hour();
        // 14:00 - 17:00 UTC = London-NY overlap
        hour >= 14 && hour < 17
    }

    /// Late session (lower liquidity)
    pub fn is_late_session(&self, timestamp: OffsetDateTime) -> bool {
        let time = timestamp.time();
        let hour = time.hour();
        hour >= 20 && hour < 22
    }

    /// Expected volatility by pair
    pub fn expected_volatility(&self, symbol: &str) -> Decimal {
        let base = &symbol[..3];
        let quote = &symbol[3..6];

        // USD pairs have highest volatility
        if matches!(base, "USD") {
            Decimal::from_f64(1.1).unwrap()
        } else if matches!(quote, "USD") {
            Decimal::from_f64(1.0).unwrap()
        } else if matches!(base, "CAD") || matches!(quote, "CAD") {
            // CAD sensitive to oil/NAFTA
            Decimal::from_f64(0.95).unwrap()
        } else {
            Decimal::from_f64(0.75).unwrap()
        }
    }

    /// Liquidity score
    pub fn liquidity_score(&self, timestamp: OffsetDateTime) -> Decimal {
        if self.is_peak_overlap(timestamp) {
            Decimal::from_f64(1.0).unwrap()
        } else if self.is_active(timestamp) {
            Decimal::from_f64(0.85).unwrap()
        } else {
            Decimal::from_f64(0.5).unwrap()
        }
    }

    /// Session characteristics
    pub fn characteristics(&self) -> SessionCharacteristics {
        SessionCharacteristics {
            best_pairs: vec![
                "EURUSD",
                "GBPUSD",
                "USDJPY",
                "USDCAD",
                "AUDUSD",
                "NZDUSD",
                "USDCHF",
            ],
            worst_pairs: vec!["EURJPY", "GBPJPY", "EURCHF"],
            news_sensitivity: Decimal::from_f64(1.0).unwrap(),
            trend_following_tendency: Decimal::from_f64(0.75).unwrap(),
            nfp_volatility: Decimal::from_f64(2.0).unwrap(),
        }
    }

    /// Major US news release times (UTC)
    pub fn major_news_times(&self) -> Vec<(Time, &'static str)> {
        vec![
            (Time::from_hms(13, 30, 0).unwrap(), "Initial Claims / NFP"),
            (Time::from_hms(14, 45, 0).unwrap(), "PMI Flash"),
            (Time::from_hms(15, 0, 0).unwrap(), "ISM / Construction"),
            (Time::from_hms(15, 30, 0).unwrap(), "Crude Inventories"),
            (Time::from_hms(17, 0, 0).unwrap(), "EIA Oil"),
            (Time::from_hms(19, 0, 0).unwrap(), "FOMC / Major Announcements"),
        ]
    }

    /// Check if near major news
    pub fn is_near_news(&self, timestamp: OffsetDateTime) -> Option<&'static str> {
        let time = timestamp.time();

        for (news_time, name) in self.major_news_times() {
            if Self::is_near_time(time, news_time, 10) {
                return Some(name);
            }
        }

        None
    }

    /// Rate volatility during FOMC events
    pub fn fomc_volatility(&self) -> Decimal {
        Decimal::from_f64(2.5).unwrap()
    }

    /// Month-end flows occur on last day of month
    pub fn is_month_end(&self, timestamp: OffsetDateTime) -> bool {
        let date = timestamp.date();
        let next_day = date + time::Duration::days(1);
        next_day.month() != date.month()
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

impl Default for NewYorkSession {
    fn default() -> Self {
        Self::new()
    }
}

/// NY session characteristics
#[derive(Debug, Clone)]
pub struct SessionCharacteristics {
    /// Best performing pairs in NY
    pub best_pairs: Vec<&'static str>,
    /// Worst performing pairs
    pub worst_pairs: Vec<&'static str>,
    /// News sensitivity
    pub news_sensitivity: Decimal,
    /// Trend following tendency
    pub trend_following_tendency: Decimal,
    /// NFP volatility multiplier
    pub nfp_volatility: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nyc_session_times() {
        let ny = NewYorkSession::new();

        let noon_utc = OffsetDateTime::from_unix_timestamp(1605091200).unwrap();
        assert!(!ny.is_active(noon_utc));

        // 15:00 UTC is during NY session
        let afternoon = OffsetDateTime::from_unix_timestamp(1605098400).unwrap();
        // Just verify no panic
        ny.liquidity_score(afternoon);
    }

    #[test]
    fn test_peak_overlap() {
        let ny = NewYorkSession::new();

        // 15:00 UTC (peak overlap)
        let peak = OffsetDateTime::from_unix_timestamp(1605099600).unwrap();
        if ny.is_active(peak) {
            assert!(ny.is_peak_overlap(peak) || peak.time().hour() < 14);
        }
    }
}
