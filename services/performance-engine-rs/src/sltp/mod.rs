use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SlTpStats {
    pub sl_hits: u32,
    pub tp_hits: u32,
    pub other_exits: u32,
}

impl SlTpStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_exit(&mut self, is_sl: bool, is_tp: bool) {
        if is_sl {
            self.sl_hits += 1;
        } else if is_tp {
            self.tp_hits += 1;
        } else {
            self.other_exits += 1;
        }
    }
}
