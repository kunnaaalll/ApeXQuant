#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueState {
    Front,
    Middle,
    Back,
    Unknown,
}

impl QueueState {
    pub fn determine(position: u64, total_queue: u64) -> Result<Self, &'static str> {
        if position > total_queue {
            return Err("Position cannot be greater than total queue");
        }
        if total_queue == 0 {
            return Ok(QueueState::Unknown);
        }

        let ratio = (position * 100) / total_queue;
        if ratio <= 25 {
            Ok(QueueState::Front)
        } else if ratio <= 75 {
            Ok(QueueState::Middle)
        } else {
            Ok(QueueState::Back)
        }
    }
}
