use serde::{Deserialize, Serialize};

use crate::health::PositionQuality;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseRecommendation {
    pub should_close: bool,
    pub reason: String,
}

pub struct CloseEngine;

impl CloseEngine {
    pub fn evaluate(
        quality: &PositionQuality,
        thesis_invalidated: bool,
        target_achieved: bool,
    ) -> CloseRecommendation {
        if thesis_invalidated {
            return CloseRecommendation {
                should_close: true,
                reason: "Original trade thesis has been invalidated.".to_string(),
            };
        }

        if *quality == PositionQuality::Critical {
            return CloseRecommendation {
                should_close: true,
                reason: "Position quality collapsed to Critical. Immediate exit required."
                    .to_string(),
            };
        }

        if target_achieved {
            return CloseRecommendation {
                should_close: true,
                reason: "Final target achieved. Closing out position.".to_string(),
            };
        }

        CloseRecommendation {
            should_close: false,
            reason: "Maintain position. No exit criteria met.".to_string(),
        }
    }
}
