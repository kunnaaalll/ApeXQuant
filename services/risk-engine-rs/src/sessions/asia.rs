use chrono::NaiveTime;

pub struct AsiaSession;

impl AsiaSession {
    pub fn is_active(time: NaiveTime) -> bool {
        let start = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        time >= start && time <= end
    }
}
