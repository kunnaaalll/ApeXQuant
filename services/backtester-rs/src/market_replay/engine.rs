use crate::market_replay::clock::{ReplayClock, ReplaySpeed};
use crate::market_replay::models::{Candle, ReplayEvent, Tick};
use std::collections::VecDeque;
use time::OffsetDateTime;

pub trait ReplayEngine {
    fn next_event(&mut self) -> Result<Option<ReplayEvent>, &'static str>;
    fn current_time(&self) -> OffsetDateTime;
    fn pause(&mut self);
    fn resume(&mut self);
    fn set_speed(&mut self, speed: ReplaySpeed);
}

pub struct TickReplayEngine {
    clock: ReplayClock,
    events: VecDeque<Tick>,
    paused: bool,
}

impl TickReplayEngine {
    pub fn new(
        start_time: OffsetDateTime,
        speed: ReplaySpeed,
        events: impl IntoIterator<Item = Tick>,
    ) -> Self {
        let mut sorted_events: Vec<Tick> = events.into_iter().collect();
        sorted_events.sort_by_key(|t| t.timestamp);

        Self {
            clock: ReplayClock::new(start_time, speed),
            events: sorted_events.into(),
            paused: false,
        }
    }
}

impl ReplayEngine for TickReplayEngine {
    fn next_event(&mut self) -> Result<Option<ReplayEvent>, &'static str> {
        if self.paused {
            return Ok(None); // Or block? For an event-driven system, returning None/yielding is better
        }

        if let Some(tick) = self.events.pop_front() {
            self.clock.advance_to(tick.timestamp)?;
            Ok(Some(ReplayEvent::Tick(tick)))
        } else {
            Ok(None)
        }
    }

    fn current_time(&self) -> OffsetDateTime {
        self.clock.current_time
    }

    fn pause(&mut self) {
        self.paused = true;
    }

    fn resume(&mut self) {
        self.paused = false;
    }

    fn set_speed(&mut self, speed: ReplaySpeed) {
        self.clock.speed = speed;
    }
}

pub struct CandleReplayEngine {
    clock: ReplayClock,
    events: VecDeque<Candle>,
    paused: bool,
}

impl CandleReplayEngine {
    pub fn new(
        start_time: OffsetDateTime,
        speed: ReplaySpeed,
        events: impl IntoIterator<Item = Candle>,
    ) -> Self {
        let mut sorted_events: Vec<Candle> = events.into_iter().collect();
        sorted_events.sort_by_key(|c| c.timestamp);

        Self {
            clock: ReplayClock::new(start_time, speed),
            events: sorted_events.into(),
            paused: false,
        }
    }
}

impl ReplayEngine for CandleReplayEngine {
    fn next_event(&mut self) -> Result<Option<ReplayEvent>, &'static str> {
        if self.paused {
            return Ok(None);
        }

        if let Some(candle) = self.events.pop_front() {
            self.clock.advance_to(candle.timestamp)?;
            Ok(Some(ReplayEvent::Candle(candle)))
        } else {
            Ok(None)
        }
    }

    fn current_time(&self) -> OffsetDateTime {
        self.clock.current_time
    }

    fn pause(&mut self) {
        self.paused = true;
    }

    fn resume(&mut self) {
        self.paused = false;
    }

    fn set_speed(&mut self, speed: ReplaySpeed) {
        self.clock.speed = speed;
    }
}

pub struct MultiSymbolReplayEngine {
    clock: ReplayClock,
    events: VecDeque<ReplayEvent>,
    paused: bool,
}

impl MultiSymbolReplayEngine {
    pub fn new(
        start_time: OffsetDateTime,
        speed: ReplaySpeed,
        events: impl IntoIterator<Item = ReplayEvent>,
    ) -> Self {
        let mut sorted_events: Vec<ReplayEvent> = events.into_iter().collect();
        sorted_events.sort_by_key(|e| e.timestamp());

        Self {
            clock: ReplayClock::new(start_time, speed),
            events: sorted_events.into(),
            paused: false,
        }
    }
}

impl ReplayEngine for MultiSymbolReplayEngine {
    fn next_event(&mut self) -> Result<Option<ReplayEvent>, &'static str> {
        if self.paused {
            return Ok(None);
        }

        if let Some(event) = self.events.pop_front() {
            self.clock.advance_to(event.timestamp())?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }

    fn current_time(&self) -> OffsetDateTime {
        self.clock.current_time
    }

    fn pause(&mut self) {
        self.paused = true;
    }

    fn resume(&mut self) {
        self.paused = false;
    }

    fn set_speed(&mut self, speed: ReplaySpeed) {
        self.clock.speed = speed;
    }
}
