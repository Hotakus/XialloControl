use tauri::AppHandle;

pub mod calibrate;
pub mod controller;
pub mod datas;
pub mod logic;
pub mod xbox;

pub fn initialize(app_handle: AppHandle) {
    controller::initialize(app_handle);
    datas::initialize();
    // controller_xbox::initialize();
}
