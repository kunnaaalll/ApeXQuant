use crate::config::Config;
use crate::signals::SignalResult;
use crate::error::Result;

pub struct SignalGenerator {}

impl SignalGenerator {
    pub fn new(_config: &Config) -> Self {
        Self {}
    }
}
