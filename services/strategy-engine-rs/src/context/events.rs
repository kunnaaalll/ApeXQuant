use crate::regime::{Regime, RegimeGrade};
use crate::session::{Session, SessionGrade};
use crate::timeframe::{Timeframe, TimeframeGrade};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextEvent {
    RegimeUpdated {
        regime: Regime,
        grade: RegimeGrade,
    },
    SessionUpdated {
        session: Session,
        grade: SessionGrade,
    },
    SymbolUpdated {
        symbol: String,
        grade: crate::symbol::SymbolGrade,
    },
    TimeframeUpdated {
        timeframe: Timeframe,
        grade: TimeframeGrade,
    },
    PatternUpdated {
        pattern: String,
        grade: crate::pattern::PatternGrade,
    },
    ContextProfileUpdated {
        profile: crate::context::StrategyContextProfile,
    },
}
