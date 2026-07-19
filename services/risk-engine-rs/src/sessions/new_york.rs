use chrono::NaiveTime;

pub struct NewYorkSession;

impl NewYorkSession {
    pub fn is_active(time: NaiveTime) -> bool {
        if let (Some(start), Some(end)) = (
            NaiveTime::from_hms_opt(13, 0, 0),
            NaiveTime::from_hms_opt(22, 0, 0),
        ) {
            time >= start && time <= end
        } else {
            false
        }
    }
}
