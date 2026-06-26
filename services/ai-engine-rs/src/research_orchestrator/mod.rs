use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResearchState {
    Queued,
    Running,
    Paused,
    Completed,
    Failed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchJob {
    pub id: String,
    pub state: ResearchState,
    pub priority_score: Decimal,
    pub target_symbol: Option<String>,
    pub target_timeframe: Option<String>,
}

pub struct ResearchJobScheduler {
    // Internal fields will manage timing and concurrency
    active_jobs: usize,
    max_concurrent_jobs: usize,
}

impl ResearchJobScheduler {
    pub fn new(max_concurrent_jobs: usize) -> Self {
        Self {
            active_jobs: 0,
            max_concurrent_jobs,
        }
    }

    pub fn can_schedule(&self) -> bool {
        self.active_jobs < self.max_concurrent_jobs
    }

    pub fn schedule(&mut self, _job: &ResearchJob) -> Result<(), &'static str> {
        if !self.can_schedule() {
            return Err("Max concurrent jobs reached");
        }
        self.active_jobs += 1;
        Ok(())
    }
    
    pub fn complete_job(&mut self) {
        if self.active_jobs > 0 {
            self.active_jobs -= 1;
        }
    }
}

pub struct ResearchQueue {
    queue: VecDeque<ResearchJob>,
}

impl ResearchQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, job: ResearchJob) {
        self.queue.push_back(job);
    }

    pub fn pop(&mut self) -> Option<ResearchJob> {
        self.queue.pop_front()
    }
}

impl Default for ResearchQueue {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ResearchPriorityEngine {}

impl ResearchPriorityEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn calculate_priority(_symbol: &str, _market_condition: &str) -> Decimal {
        // Placeholder for priority calculation
        Decimal::new(50, 0)
    }

    pub fn reorder_queue(queue: &mut ResearchQueue) {
        // Sort queue by priority score descending
        queue.queue.make_contiguous().sort_by_key(|b| std::cmp::Reverse(b.priority_score));
    }
}

impl Default for ResearchPriorityEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ResearchLifecycleManager {
    scheduler: ResearchJobScheduler,
    queue: ResearchQueue,
}

impl ResearchLifecycleManager {
    pub fn new(max_concurrent_jobs: usize) -> Self {
        Self {
            scheduler: ResearchJobScheduler::new(max_concurrent_jobs),
            queue: ResearchQueue::new(),
        }
    }

    pub fn submit_job(&mut self, mut job: ResearchJob) {
        job.state = ResearchState::Queued;
        self.queue.push(job);
        ResearchPriorityEngine::reorder_queue(&mut self.queue);
    }

    pub fn tick(&mut self) {
        while self.scheduler.can_schedule() {
            if let Some(mut job) = self.queue.pop() {
                job.state = ResearchState::Running;
                if self.scheduler.schedule(&job).is_ok() {
                    // Job is now running (simulated)
                } else {
                    // Should not happen since we checked can_schedule
                    job.state = ResearchState::Queued;
                    self.queue.push(job);
                }
            } else {
                break;
            }
        }
    }
}
