use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorrelationWindow {
    ShortTerm,
    MediumTerm,
    LongTerm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorrelationType {
    Symbol,
    Currency,
    Sector,
    Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    pub matrix_type: CorrelationType,
    pub window: CorrelationWindow,
    pub identifiers: Vec<String>,
    pub data: Vec<Decimal>,
    pub rows: usize,
    pub cols: usize,
}

impl CorrelationMatrix {
    pub fn new(matrix_type: CorrelationType, window: CorrelationWindow, identifiers: Vec<String>) -> Self {
        let size = identifiers.len();
        let mut data = vec![Decimal::ZERO; size * size];
        
        // Initialize diagonal to 1.0
        for i in 0..size {
            data[i * size + i] = Decimal::ONE;
        }

        Self {
            matrix_type,
            window,
            identifiers,
            data,
            rows: size,
            cols: size,
        }
    }

    pub fn get_correlation(&self, idx_a: usize, idx_b: usize) -> Option<Decimal> {
        if idx_a < self.rows && idx_b < self.cols {
            Some(self.data[idx_a * self.cols + idx_b])
        } else {
            None
        }
    }

    pub fn set_correlation(&mut self, idx_a: usize, idx_b: usize, value: Decimal) {
        if idx_a < self.rows && idx_b < self.cols {
            self.data[idx_a * self.cols + idx_b] = value;
            self.data[idx_b * self.cols + idx_a] = value; // Symmetric matrix
        }
    }
}
