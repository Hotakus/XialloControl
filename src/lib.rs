use std::env;
use tauri::Window;
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

mod controller;
mod tray;
mod setting;
mod xeno_utils;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn close_current_window(window: Window) -> Result<(), String> {
    window.close().map_err(|e| format!("关闭窗口失败: {}", e))
}

#[tauri::command]
fn minimize_current_window(window: Window) -> Result<(), String> {
    window
        .minimize()
        .map_err(|e| format!("最小化窗口失败: {}", e))
}


#[tauri::command]
fn open_url(url: &str) -> Result<(), String> {
    tauri_plugin_opener::open_url(url, None::<&str>).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .invoke_handler(tauri::generate_handler![
            greet,
            close_current_window,
            minimize_current_window,
            open_url,

            controller::query_devices,
            controller::use_device,
            controller::disconnect_device,
            controller::set_frequency,

            setting::get_current_settings,
            setting::update_settings
        ])
        .setup(|app| {
            let app_handle = app.handle();
            let _ = tray::initialize(app_handle.clone());
            let _ = controller::initialize(app_handle.clone());
            let _ = xeno_utils::initialize();
            let _ = setting::initialize();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
