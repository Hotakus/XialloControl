use crate::{adaptive_sampler, controller, mapping, preset, setting, xeno_utils};
use log::Level;
use tauri::AppHandle;
use tauri_plugin_log::fern::colors::{Color, ColoredLevelConfig};

pub fn initialize() {
    setting::initialize();
    preset::initialize();

    mapping::initialize();

    adaptive_sampler::initialize();
}
