#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicrostructureGrade {
    Elite,
    Strong,
    Normal,
    Weak,
    Poor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MicrostructureScore {
    pub score: u8, // 0-100
    pub grade: MicrostructureGrade,
}

impl MicrostructureScore {
    pub fn calculate(
        spread_score: u8,
        imbalance_score: u8,
        depth_score: u8,
        resiliency_score: u8,
        volatility_score: u8,
        cost_score: u8,
    ) -> Result<Self, &'static str> {
        if spread_score > 100
            || imbalance_score > 100
            || depth_score > 100
            || resiliency_score > 100
            || volatility_score > 100
            || cost_score > 100
        {
            return Err("Scores must be bounded between 0 and 100");
        }

        let total = spread_score as u16
            + imbalance_score as u16
            + depth_score as u16
            + resiliency_score as u16
            + volatility_score as u16
            + cost_score as u16;

        let avg = (total / 6) as u8;

        let grade = if avg >= 90 {
            MicrostructureGrade::Elite
        } else if avg >= 75 {
            MicrostructureGrade::Strong
        } else if avg >= 50 {
            MicrostructureGrade::Normal
        } else if avg >= 25 {
            MicrostructureGrade::Weak
        } else {
            MicrostructureGrade::Poor
        };

        Ok(Self { score: avg, grade })
    }
}
