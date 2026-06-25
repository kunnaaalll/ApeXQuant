use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StructureState {
    Trending,
    Ranging,
    Expansion,
    Compression,
    Transition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructureMetrics {
    pub last_swing_high: Decimal,
    pub last_swing_low: Decimal,
    pub break_of_structure_up: bool,
    pub break_of_structure_down: bool,
    pub market_structure_shift_up: bool,
    pub market_structure_shift_down: bool,
    pub trend_continuation: bool,
    pub state: StructureState,
}

#[derive(Debug, Clone)]
pub struct StructureEngine {
    window: usize,
    highs: Vec<Decimal>,
    lows: Vec<Decimal>,
    last_swing_high: Decimal,
    last_swing_low: Decimal,
    current_trend_dir: i8, // 1 up, -1 down
}

impl StructureEngine {
    pub fn new(window: usize) -> Self {
        Self {
            window,
            highs: Vec::with_capacity(window),
            lows: Vec::with_capacity(window),
            last_swing_high: Decimal::ZERO,
            last_swing_low: Decimal::MAX,
            current_trend_dir: 0,
        }
    }

    pub fn update(&mut self, high: Decimal, low: Decimal) -> Result<StructureMetrics, &'static str> {
        if high < low {
            return Err("High cannot be less than low");
        }

        self.highs.push(high);
        self.lows.push(low);

        if self.highs.len() > self.window {
            self.highs.remove(0);
            self.lows.remove(0);
        }

        let mut local_max = Decimal::ZERO;
        let mut local_min = Decimal::MAX;
        
        for h in &self.highs {
            if *h > local_max {
                local_max = *h;
            }
        }
        for l in &self.lows {
            if *l < local_min {
                local_min = *l;
            }
        }

        let mut bos_up = false;
        let mut bos_down = false;
        let mut mss_up = false;
        let mut mss_down = false;

        if local_max > self.last_swing_high && !self.last_swing_high.is_zero() {
            if self.current_trend_dir == 1 {
                bos_up = true;
            } else if self.current_trend_dir == -1 {
                mss_up = true;
                self.current_trend_dir = 1;
            }
            self.last_swing_high = local_max;
        }

        if local_min < self.last_swing_low && self.last_swing_low != Decimal::MAX {
            if self.current_trend_dir == -1 {
                bos_down = true;
            } else if self.current_trend_dir == 1 {
                mss_down = true;
                self.current_trend_dir = -1;
            }
            self.last_swing_low = local_min;
        }

        if self.last_swing_high.is_zero() {
            self.last_swing_high = local_max;
        }
        if self.last_swing_low == Decimal::MAX {
            self.last_swing_low = local_min;
        }

        let trend_continuation = bos_up || bos_down;
        
        let range = self.last_swing_high - self.last_swing_low;
        let state = if trend_continuation {
            StructureState::Trending
        } else if mss_up || mss_down {
            StructureState::Transition
        } else if range > (self.last_swing_low * Decimal::from_str_exact("0.05").unwrap_or(Decimal::ZERO)) {
            StructureState::Expansion
        } else if range > Decimal::ZERO {
            StructureState::Ranging
        } else {
            StructureState::Compression
        };

        Ok(StructureMetrics {
            last_swing_high: self.last_swing_high,
            last_swing_low: self.last_swing_low,
            break_of_structure_up: bos_up,
            break_of_structure_down: bos_down,
            market_structure_shift_up: mss_up,
            market_structure_shift_down: mss_down,
            trend_continuation,
            state,
        })
    }
}
