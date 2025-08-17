use crate::{adaptive_sampler, controller, mapping, preset, setting, xeno_utils};
use tauri::AppHandle;

pub fn initialize(app_handle: AppHandle) {
    xeno_utils::initialize();

    setting::initialize();

    preset::initialize();

    mapping::initialize();
    controller::initialize(app_handle.clone());

    adaptive_sampler::initialize();
}
