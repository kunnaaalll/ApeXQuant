use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsychologicalProfile {
    pub average_fomo_score: f64,
    pub discipline_rating: f64,
}

impl PsychologicalProfile {
    pub fn calculate(fomo_signals: &[f64], discipline_signals: &[f64]) -> Self {
        let fomo_avg = if fomo_signals.is_empty() {
            0.0
        } else {
            fomo_signals.iter().sum::<f64>() / fomo_signals.len() as f64
        };

        let disc_avg = if discipline_signals.is_empty() {
            1.0
        } else {
            discipline_signals.iter().sum::<f64>() / discipline_signals.len() as f64
        };

        Self {
            average_fomo_score: fomo_avg,
            discipline_rating: disc_avg,
        }
    }
}
