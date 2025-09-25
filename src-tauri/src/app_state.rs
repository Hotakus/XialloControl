use std::sync::Arc;

use crate::preset::PresetManager;

#[derive(Debug)]
pub struct AppState {
    pub preset_manager: PresetManager,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            preset_manager: PresetManager::default(),
        }
    }
}

pub struct AppStateWrapper {
    pub app_state: Arc<AppState>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            preset_manager: PresetManager::new(),
        }
    }
}
