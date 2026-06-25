use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrendState {
    StrongBullish,
    Bullish,
    Neutral,
    Bearish,
    StrongBearish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrendMetrics {
    pub direction: i8, // 1 for up, -1 for down, 0 for neutral
    pub strength_score: u8, // 0-100
    pub persistence: u32, // number of ticks/candles in current direction
    pub is_accelerating: bool,
    pub is_exhausted: bool,
    pub state: TrendState,
}

#[derive(Debug, Clone)]
pub struct TrendEngine {
    window_fast: usize,
    window_slow: usize,
    prices: Vec<Decimal>,
    persistence: u32,
    last_direction: i8,
    strength_history: Vec<Decimal>,
}

impl TrendEngine {
    pub fn new(window_fast: usize, window_slow: usize) -> Self {
        Self {
            window_fast,
            window_slow,
            prices: Vec::with_capacity(window_slow),
            persistence: 0,
            last_direction: 0,
            strength_history: Vec::with_capacity(10),
        }
    }

    pub fn update(&mut self, price: Decimal) -> Result<TrendMetrics, &'static str> {
        self.prices.push(price);
        if self.prices.len() > self.window_slow {
            self.prices.remove(0);
        }

        let fast_len = self.prices.len().min(self.window_fast);
        let slow_len = self.prices.len();

        if fast_len == 0 {
            return Ok(TrendMetrics {
                direction: 0,
                strength_score: 0,
                persistence: 0,
                is_accelerating: false,
                is_exhausted: false,
                state: TrendState::Neutral,
            });
        }

        let fast_sum: Decimal = self.prices[self.prices.len() - fast_len..].iter().sum();
        let sma_fast = fast_sum / Decimal::from(fast_len);

        let slow_sum: Decimal = self.prices.iter().sum();
        let sma_slow = slow_sum / Decimal::from(slow_len);

        let distance = if sma_slow.is_zero() {
            Decimal::ZERO
        } else {
            (sma_fast - sma_slow) / sma_slow * Decimal::from(10000)
        };

        let current_direction = if distance > Decimal::from(10) {
            1
        } else if distance < Decimal::from(-10) {
            -1
        } else {
            0
        };

        if current_direction == self.last_direction {
            self.persistence += 1;
        } else {
            self.persistence = 1;
            self.last_direction = current_direction;
        }

        let raw_strength = distance.abs();
        self.strength_history.push(raw_strength);
        if self.strength_history.len() > 5 {
            self.strength_history.remove(0);
        }

        let mut is_accelerating = false;
        let mut is_exhausted = false;
        
        if self.strength_history.len() >= 3 {
            let len = self.strength_history.len();
            let s1 = self.strength_history[len - 1];
            let s2 = self.strength_history[len - 2];
            let s3 = self.strength_history[len - 3];
            
            if s1 > s2 && s2 > s3 {
                is_accelerating = true;
            }
            if s1 < s2 && s2 < s3 && self.persistence > 20 {
                is_exhausted = true;
            }
        }

        // Cap strength score 0-100
        let capped = if raw_strength > Decimal::from(500) {
            Decimal::from(100)
        } else {
            raw_strength / Decimal::from(5)
        };
        
        let strength_score = capped.to_u8().unwrap_or(100).min(100);

        let state = match (current_direction, strength_score) {
            (1, s) if s >= 70 => TrendState::StrongBullish,
            (1, s) if s >= 30 => TrendState::Bullish,
            (-1, s) if s >= 70 => TrendState::StrongBearish,
            (-1, s) if s >= 30 => TrendState::Bearish,
            _ => TrendState::Neutral,
        };

        Ok(TrendMetrics {
            direction: current_direction,
            strength_score,
            persistence: self.persistence,
            is_accelerating,
            is_exhausted,
            state,
        })
    }
}
