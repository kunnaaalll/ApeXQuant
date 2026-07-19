use chrono::NaiveTime;

pub struct AsiaSession;

impl AsiaSession {
    pub fn is_active(time: NaiveTime) -> bool {
        if let (Some(start), Some(end)) = (
            NaiveTime::from_hms_opt(0, 0, 0),
            NaiveTime::from_hms_opt(9, 0, 0),
        ) {
            time >= start && time <= end
        } else {
            false
        }
    }
}
