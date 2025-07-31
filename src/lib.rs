// "windows": [
//       {
//         "label": "main",
//         "title": "XenoControl",
//         "fullscreen": false,
//         "decorations": true,
//         "width": 1080,
//         "height": 720,
//         "resizable": false,
//         "transparent": false,
//         "maximizable": false
//       }
//     ],

use std::env;
use tauri::{Window}; // 👈 引入 Manager 以启用 create_window
use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::{MacosLauncher};

mod controller;
mod setting;
mod tray;
mod xeno_utils;
mod adaptive_sampler;
mod mapping;

#[tauri::command]
fn hide_current_window(window: Window) -> Result<(), String> {
    window.hide().map_err(|e| format!("隐藏窗口失败: {}", e))
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

#[tauri::command]
fn get_platform() -> String {
    std::env::consts::OS.to_string() // "linux" | "windows" | "macos" 等
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .invoke_handler(tauri::generate_handler![
            hide_current_window,
            close_current_window,
            minimize_current_window,
            open_url,
            get_platform,

            controller::controller::query_devices,
            controller::controller::use_device,
            controller::controller::disconnect_device,
            controller::controller::set_frequency,
            controller::controller::get_controller_data,
            controller::logic::controller_stick_drift_sampling,

            setting::get_current_settings,
            setting::update_settings,
            mapping::set_mapping,
            mapping::get_mappings
        ])
        .setup(|app| {
            let app_handle = app.handle();

            // 判断是否为 Windows 平台
            #[cfg(target_os = "windows")]
            let decorations = false;
            #[cfg(not(target_os = "windows"))]
            let decorations = true;

            WebviewWindowBuilder::new(
                &app_handle.clone(),
                "main",
                WebviewUrl::App("index.html".into()),
            )
            .title("XenoControl")
            .inner_size(1080.0, 720.0)
            .resizable(false)
            .fullscreen(false)
            .decorations(decorations)
            .transparent(false)
            .maximizable(false)
            .build()?;

            let _ = adaptive_sampler::initialize();
            let _ = tray::initialize(app_handle.clone());
            let _ = controller::initialize(app_handle.clone());
            // let _ = controller_datas::initialize();
            let _ = xeno_utils::initialize();
            let _ = setting::initialize();
            let _ = mapping::initialize();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
