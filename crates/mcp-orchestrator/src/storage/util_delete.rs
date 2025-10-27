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
}
