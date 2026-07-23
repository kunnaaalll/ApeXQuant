//! Market data ingestion and management

pub mod buffer;
pub mod candle;
pub mod atr;
pub mod validator;

pub use buffer::CandleBuffer;
pub use candle::Candle;
pub use atr::calculate_atr;
pub use validator::DataValidator;
