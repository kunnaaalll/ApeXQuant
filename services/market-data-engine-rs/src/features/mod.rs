pub mod extraction {
    use rust_decimal::Decimal;

    pub trait FeatureExtractor {
        fn extract(&self, inputs: &[Decimal]) -> Result<Decimal, &'static str>;
    }
}

pub mod types;
pub mod store;

pub use types::*;
pub use store::*;
