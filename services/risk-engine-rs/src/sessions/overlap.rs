use chrono::NaiveTime;

pub struct SessionOverlap;

impl SessionOverlap {
    pub fn is_asia_london(time: NaiveTime) -> bool {
        if let (Some(start), Some(end)) = (
            NaiveTime::from_hms_opt(8, 0, 0),
            NaiveTime::from_hms_opt(9, 0, 0),
        ) {
            time >= start && time <= end
        } else {
            false
        }
    }

    pub fn is_london_new_york(time: NaiveTime) -> bool {
        if let (Some(start), Some(end)) = (
            NaiveTime::from_hms_opt(13, 0, 0),
            NaiveTime::from_hms_opt(17, 0, 0),
        ) {
            time >= start && time <= end
        } else {
            false
        }
    }
}
