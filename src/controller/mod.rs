use tauri::AppHandle;

pub mod controller;
pub mod datas;
pub mod xbox;
pub mod logic;
mod calibrate;

pub fn initialize(app_handle: AppHandle) {
    controller::initialize(app_handle);
    datas::initialize();
    // controller_xbox::initialize();
}
