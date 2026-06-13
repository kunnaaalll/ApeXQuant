//! Signal grading engine

use crate::confluence::ConfluenceScore;

/// Grade a confluence score
pub struct GradingEngine;

impl GradingEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn grade(&self, score: &ConfluenceScore) -> Grade {
        if score.total >= 85.0 {
            Grade::APlus
        } else if score.total >= 70.0 {
            Grade::A
        } else if score.total >= 60.0 {
            Grade::B
        } else {
            Grade::Reject
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    APlus,
    A,
    B,
    Reject,
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Grade::APlus => write!(f, "A+"),
            Grade::A => write!(f, "A"),
            Grade::B => write!(f, "B"),
            Grade::Reject => write!(f, "REJECT"),
        }
    }
}
