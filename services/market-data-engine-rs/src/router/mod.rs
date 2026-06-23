#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterMode {
    Primary,
    Secondary,
    Standby,
}

pub struct FeedRouter {
    pub primary_feed_id: Option<String>,
    pub secondary_feed_id: Option<String>,
    pub standby_feed_id: Option<String>,
    pub current_mode: RouterMode,
}

impl Default for FeedRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedRouter {
    pub fn new() -> Self {
        Self {
            primary_feed_id: None,
            secondary_feed_id: None,
            standby_feed_id: None,
            current_mode: RouterMode::Primary,
        }
    }

    pub fn set_primary(&mut self, id: String) {
        self.primary_feed_id = Some(id);
    }

    pub fn set_secondary(&mut self, id: String) {
        self.secondary_feed_id = Some(id);
    }

    pub fn set_standby(&mut self, id: String) {
        self.standby_feed_id = Some(id);
    }

    pub fn switch_mode(&mut self, mode: RouterMode) {
        self.current_mode = mode;
    }

    pub fn active_feed_id(&self) -> Option<String> {
        match self.current_mode {
            RouterMode::Primary => self.primary_feed_id.clone(),
            RouterMode::Secondary => self.secondary_feed_id.clone(),
            RouterMode::Standby => self.standby_feed_id.clone(),
        }
    }
}
