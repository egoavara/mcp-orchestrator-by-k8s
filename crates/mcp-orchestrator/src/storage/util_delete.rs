use chrono::Duration;

pub enum DeleteResult {
    Deleted,
    Deleting,
}

#[derive(Clone, Default)]
pub struct DeleteOption {
    pub remove_finalizer: Option<bool>,
    pub timeout: Option<Duration>,
}

impl DeleteOption {
    pub fn timeout(duration: Duration) -> Self {
        Self {
            timeout: Some(duration),
            ..Default::default()
        }
    }
    pub fn timeout_millis(millis: i64) -> Self {
        Self {
            timeout: Some(Duration::milliseconds(millis)),
            ..Default::default()
        }
    }

    pub fn remove_finalizer() -> Self {
        Self {
            remove_finalizer: Some(true),
            ..Default::default()
        }
    }

    pub fn with_force(mut self, force: bool) -> Self {
        self.remove_finalizer = Some(force);
        self
    }

    pub fn with_timeout_millis(mut self, millis: i64) -> Self {
        self.timeout = Some(Duration::milliseconds(millis));
        self
    }
}
