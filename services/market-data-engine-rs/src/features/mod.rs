pub mod extraction {
    use rust_decimal::Decimal;

    pub trait FeatureExtractor {
        fn extract(&self, inputs: &[Decimal]) -> Result<Decimal, &'static str>;
    }
}

pub mod store;
pub mod types;

pub use store::*;
pub use types::*;
