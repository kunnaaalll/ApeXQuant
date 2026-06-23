#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TradingSession {
    Asia,
    London,
    NewYork,
    Overlap,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionQuality {
    Excellent,
    Good,
    Normal,
    Poor,
}

pub struct SessionEngine;

impl SessionEngine {
    pub fn evaluate(session: TradingSession) -> Result<SessionQuality, &'static str> {
        let quality = match session {
            TradingSession::Overlap => SessionQuality::Excellent,
            TradingSession::NewYork | TradingSession::London => SessionQuality::Good,
            TradingSession::Asia => SessionQuality::Normal,
            TradingSession::Closed => SessionQuality::Poor,
        };
        Ok(quality)
    }
}
