use serde::{Deserialize, Serialize};

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
    pub data: Vec<f64>,
    pub rows: usize,
    pub cols: usize,
}

impl CorrelationMatrix {
    pub fn new(matrix_type: CorrelationType, window: CorrelationWindow, identifiers: Vec<String>) -> Self {
        let size = identifiers.len();
        let mut data = vec![0.0; size * size];
        
        // Initialize diagonal to 1.0
        for i in 0..size {
            data[i * size + i] = 1.0;
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

    pub fn get_correlation(&self, idx_a: usize, idx_b: usize) -> Option<f64> {
        if idx_a < self.rows && idx_b < self.cols {
            Some(self.data[idx_a * self.cols + idx_b])
        } else {
            None
        }
    }

    pub fn set_correlation(&mut self, idx_a: usize, idx_b: usize, value: f64) {
        if idx_a < self.rows && idx_b < self.cols {
            self.data[idx_a * self.cols + idx_b] = value;
            self.data[idx_b * self.cols + idx_a] = value; // Symmetric matrix
        }
    }
}
