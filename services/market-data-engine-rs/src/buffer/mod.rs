use std::collections::VecDeque;
use crate::tick::Tick;

pub struct TickBuffer {
    capacity: usize,
    buffer: VecDeque<Tick>,
}

impl TickBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            buffer: VecDeque::with_capacity(capacity),
        }
    }

    pub fn append(&mut self, tick: Tick) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(tick);
    }

    pub fn lookup(&self, sequence: u64) -> Option<&Tick> {
        self.buffer.iter().find(|t| t.sequence == sequence)
    }

    pub fn oldest(&self) -> Option<&Tick> {
        self.buffer.front()
    }

    pub fn latest(&self) -> Option<&Tick> {
        self.buffer.back()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tick> {
        self.buffer.iter()
    }
}
