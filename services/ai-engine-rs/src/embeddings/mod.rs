use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingVector {
    pub values: Vec<f32>,
}

impl EmbeddingVector {
    pub fn new(values: Vec<f32>) -> Self {
        Self { values }
    }

    pub fn cosine_similarity(&self, other: &EmbeddingVector) -> f32 {
        if self.values.len() != other.values.len() || self.values.is_empty() {
            return 0.0;
        }

        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for i in 0..self.values.len() {
            dot_product += self.values[i] * other.values[i];
            norm_a += self.values[i] * self.values[i];
            norm_b += other.values[i] * other.values[i];
        }

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a.sqrt() * norm_b.sqrt())
    }
}

pub struct EmbeddingsEngine;

impl EmbeddingsEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_embedding(&self, text: &str) -> EmbeddingVector {
        // Deterministic pseudo-embedding for zero-trust implementation without a real neural net
        // Generates a 32-dim vector based on string bytes
        let mut values = vec![0.0; 32];
        for (i, byte) in text.bytes().enumerate() {
            let pos = i % 32;
            values[pos] += ((byte as f32) * ((i as f32) + 1.0)) / 255.0;
            // Add a cross-term to spread variance
            values[(pos + 7) % 32] -= ((byte as f32) / 128.0).sin();
        }
        
        // Normalize
        let mut norm = 0.0;
        for val in &values {
            norm += val * val;
        }
        if norm > 0.0 {
            let sqrt_norm = norm.sqrt();
            for val in &mut values {
                *val /= sqrt_norm;
            }
        }

        EmbeddingVector::new(values)
    }
}
