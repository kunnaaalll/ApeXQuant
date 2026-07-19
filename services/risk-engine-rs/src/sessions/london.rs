use chrono::NaiveTime;

pub struct LondonSession;

impl LondonSession {
    pub fn is_active(time: NaiveTime) -> bool {
        if let (Some(start), Some(end)) = (
            NaiveTime::from_hms_opt(8, 0, 0),
            NaiveTime::from_hms_opt(17, 0, 0),
        ) {
            time >= start && time <= end
        } else {
            false
        }
    }
}
