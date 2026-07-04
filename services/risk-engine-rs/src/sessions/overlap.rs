use chrono::NaiveTime;

pub struct SessionOverlap;

impl SessionOverlap {
    pub fn is_asia_london(time: NaiveTime) -> bool {
        let start = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        time >= start && time <= end
    }

    pub fn is_london_new_york(time: NaiveTime) -> bool {
        let start = NaiveTime::from_hms_opt(13, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
        time >= start && time <= end
    }
}
