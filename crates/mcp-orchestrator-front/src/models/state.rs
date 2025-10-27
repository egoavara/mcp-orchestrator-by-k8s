use serde::{Deserialize, Serialize};
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "session", storage_tab_sync)]
pub struct SessionState {
    pub selected_namespace: Option<String>,
    pub breadcrumbs: Vec<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "local")]
pub struct UserPreferences {
    pub theme: Theme,
    pub items_per_page: usize,
    pub default_namespace: Option<String>,
    pub show_deleted: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            items_per_page: 20,
            default_namespace: None,
            show_deleted: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub enum Theme {
    Light,
    #[default]
    Dark,
}

