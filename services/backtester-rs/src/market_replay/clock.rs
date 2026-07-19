use core::time::Duration as StdDuration;
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaySpeed {
    OneX,
    TenX,
    OneHundredX,
    OneThousandX,
    Unlimited,
}

#[derive(Debug, Clone)]
pub struct ReplayClock {
    pub current_time: OffsetDateTime,
    pub speed: ReplaySpeed,
}

impl ReplayClock {
    pub fn new(start_time: OffsetDateTime, speed: ReplaySpeed) -> Self {
        Self {
            current_time: start_time,
            speed,
        }
    }

    pub fn advance_to(&mut self, target_time: OffsetDateTime) -> Result<(), &'static str> {
        if target_time < self.current_time {
            return Err("Cannot advance backwards in time");
        }

        let delta = target_time - self.current_time;
        // In Unlimited mode, simulated time jumps directly.
        // In other modes, we could add real sleep here, but for a deterministic engine,
        // we often just advance simulated time and maybe throttle real time elsewhere.
        // For now, we will simply jump the simulated time. Real-time throttling can be
        // managed at the execution layer or engine layer.

        // Example logic for real-time throttling if needed:
        if self.speed != ReplaySpeed::Unlimited {
            let speed_factor = match self.speed {
                ReplaySpeed::OneX => 1,
                ReplaySpeed::TenX => 10,
                ReplaySpeed::OneHundredX => 100,
                ReplaySpeed::OneThousandX => 1000,
                ReplaySpeed::Unlimited => 1, // Fallback, though guarded by the if condition
            };

            let real_micros = delta.whole_microseconds() as u64 / speed_factor;
            if real_micros > 0 {
                std::thread::sleep(StdDuration::from_micros(real_micros));
            }
        }

        self.current_time = target_time;
        Ok(())
    }
}
