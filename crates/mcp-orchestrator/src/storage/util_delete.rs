use chrono::Duration;

pub enum DeleteResult {
    Deleted,
    Deleting,
}

#[derive(Clone, Default)]
pub struct DeleteOption {
    // TODO: finalizer 제거하고 owner_references 처리로 변경
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
