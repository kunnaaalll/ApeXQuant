#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceState {
    Healthy,
    Warning,
    Broken,
}

pub struct SequenceTracker {
    last_sequence: Option<u64>,
    state: SequenceState,
}

impl Default for SequenceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl SequenceTracker {
    pub fn new() -> Self {
        Self {
            last_sequence: None,
            state: SequenceState::Healthy,
        }
    }

    pub fn state(&self) -> SequenceState {
        self.state
    }

    pub fn process_sequence(&mut self, sequence: u64) -> SequenceResult {
        match self.last_sequence {
            None => {
                self.last_sequence = Some(sequence);
                self.state = SequenceState::Healthy;
                SequenceResult::Ok
            }
            Some(last) => {
                if sequence == last + 1 {
                    self.last_sequence = Some(sequence);
                    self.state = SequenceState::Healthy;
                    SequenceResult::Ok
                } else if sequence <= last {
                    self.state = SequenceState::Warning;
                    SequenceResult::DuplicateOrOutOfOrder
                } else {
                    let missing = sequence - last - 1;
                    if missing > 100 {
                        self.state = SequenceState::Broken;
                    } else {
                        self.state = SequenceState::Warning;
                    }
                    self.last_sequence = Some(sequence);
                    SequenceResult::Missing(missing)
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.last_sequence = None;
        self.state = SequenceState::Healthy;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceResult {
    Ok,
    DuplicateOrOutOfOrder,
    Missing(u64),
}
