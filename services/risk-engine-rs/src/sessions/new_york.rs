use chrono::NaiveTime;

pub struct NewYorkSession;

impl NewYorkSession {
    pub fn is_active(time: NaiveTime) -> bool {
        let start = NaiveTime::from_hms_opt(13, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        time >= start && time <= end
    }
}
