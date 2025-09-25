use std::sync::Arc;

use tauri::AppHandle;

use crate::{adaptive_sampler, app_state::AppState, mapping, preset, setting, controller};

pub fn initialize(app_handle: AppHandle, app_state: Arc<AppState>) {
    setting::initialize();
    preset::initialize(app_state.clone());

    mapping::initialize();

    adaptive_sampler::initialize();

    controller::initialize(app_handle.clone());
}
