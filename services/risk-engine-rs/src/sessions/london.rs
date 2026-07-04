use chrono::NaiveTime;

pub struct LondonSession;

impl LondonSession {
    pub fn is_active(time: NaiveTime) -> bool {
        let start = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
        time >= start && time <= end
    }
}
