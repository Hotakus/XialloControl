use tauri::AppHandle;

pub mod calibrate;
pub mod controller;
pub mod datas;
pub mod logic;
pub mod xbox;
pub mod ps4;

pub fn initialize(app_handle: AppHandle) {
    controller::initialize(app_handle);
    datas::initialize();
    ps4::initialize();
    // controller_xbox::initialize();
}
