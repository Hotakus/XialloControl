use tauri::AppHandle;

pub mod controller;
pub mod controller_datas;
pub mod controller_xbox;


pub fn initialize(app_handle: AppHandle) {
    controller::initialize(app_handle);
    controller_datas::initialize();
    // controller_xbox::initialize();
}
