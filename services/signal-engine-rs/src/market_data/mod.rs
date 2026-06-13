//! Market data ingestion and management

pub mod buffer;
pub mod candle;
pub mod validator;

pub use buffer::CandleBuffer;
pub use candle::Candle;
pub use validator::DataValidator;
