#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticipationGrade {
    High,
    Normal,
    Low,
    VeryLow,
}

impl ParticipationGrade {
    pub fn evaluate(participation_rate: u8) -> Result<Self, &'static str> {
        if participation_rate > 100 {
            return Err("Participation rate must be bounded between 0 and 100");
        }

        if participation_rate >= 80 {
            Ok(ParticipationGrade::High)
        } else if participation_rate >= 40 {
            Ok(ParticipationGrade::Normal)
        } else if participation_rate >= 10 {
            Ok(ParticipationGrade::Low)
        } else {
            Ok(ParticipationGrade::VeryLow)
        }
    }
}
