use chrono::{DateTime, Utc, NaiveTime};
use super::asia::AsiaSession;
use super::london::LondonSession;
use super::new_york::NewYorkSession;
use super::overlap::SessionOverlap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradingSession {
    Asia,
    London,
    NewYork,
    AsiaLondonOverlap,
    LondonNewYorkOverlap,
    OffHours,
}

pub struct SessionClassifier;

impl SessionClassifier {
    pub fn classify(dt: DateTime<Utc>) -> TradingSession {
        let time = dt.time();

        if SessionOverlap::is_london_new_york(time) {
            TradingSession::LondonNewYorkOverlap
        } else if SessionOverlap::is_asia_london(time) {
            TradingSession::AsiaLondonOverlap
        } else if NewYorkSession::is_active(time) {
            TradingSession::NewYork
        } else if LondonSession::is_active(time) {
            TradingSession::London
        } else if AsiaSession::is_active(time) {
            TradingSession::Asia
        } else {
            TradingSession::OffHours
        }
    }
}
