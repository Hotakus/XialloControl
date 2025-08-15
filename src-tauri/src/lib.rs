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
use tauri::{AppHandle, Manager, WebviewWindow, Window};
// 👈 引入 Manager 以启用 create_window
use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::MacosLauncher;

mod adaptive_sampler;
mod controller;
mod mapping;
mod preset;
mod setting;
mod tray;
mod xeno_utils;

#[tauri::command]
fn hide_current_window(window: Window) -> Result<(), String> {
    window.hide().map_err(|e| format!("隐藏窗口失败: {e}"))
}

#[tauri::command]
fn close_current_window(window: Window) -> Result<(), String> {
    window.close().map_err(|e| format!("关闭窗口失败: {e}"))
}

#[tauri::command]
fn minimize_current_window(window: Window) -> Result<(), String> {
    window
        .minimize()
        .map_err(|e| format!("最小化窗口失败: {e}"))
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

#[tauri::command]
fn _create_child_window(app_handle: AppHandle) {
    let child_window = app_handle.get_webview_window("child");
    child_window
        .unwrap()
        .show()
        .unwrap_or_else(|e| log::error!("Failed to show child window: {e}"));
}

fn create_child_window(app_handle: AppHandle) -> WebviewWindow {
    // 判断是否为 Windows 平台
    #[cfg(target_os = "windows")]
    let decorations = false;
    #[cfg(not(target_os = "windows"))]
    let decorations = true;

    WebviewWindowBuilder::new(
        &app_handle.clone(),
        "child",
        WebviewUrl::App("child.html".into()),
    )
    .title("child")
    .inner_size(800.0, 620.0)
    .resizable(false)
    .fullscreen(false)
    .decorations(decorations)
    .transparent(false)
    .maximizable(false)
    .visible(false)
    .build()
    .expect("Failed to create child window")
}

fn create_main_window(app_handle: AppHandle) -> WebviewWindow {
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
    .min_inner_size(1130.0, 770.0)
    .resizable(true)
    .fullscreen(false)
    .decorations(decorations)
    .transparent(true)
    .maximizable(false)
    .build()
    .expect("Failed to create main window")
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
            _create_child_window,

            controller::controller::query_devices,
            controller::controller::use_device,
            controller::controller::disconnect_device,
            controller::controller::set_frequency,
            controller::controller::get_controller_data,
            controller::logic::controller_stick_drift_sampling,
            controller::logic::check_controller_deadzone,

            setting::get_current_settings,
            setting::update_settings,

            mapping::set_mapping,
            mapping::get_mappings,
            mapping::update_mapping,
            mapping::add_mapping,
            mapping::delete_mapping,

            preset::preset_test,
            preset::preset_test2,
            preset::get_preset
        ])
        .setup(|app| {
            let app_handle = app.handle();

            let main_window = create_main_window(app_handle.clone());
            let child_window = create_child_window(app_handle.clone());

            let child_windows_list = vec![
                child_window.clone(),
            ];

            main_window.on_window_event({
                let app_handle = app_handle.clone(); // 闭包再 clone 一次，保证 'static
                move |event| {
                    if let tauri::WindowEvent::CloseRequested { api: _, .. } = event {
                        log::info!("Main window close requested");
                        for w in &child_windows_list {
                            w.close().unwrap_or_else(|e| log::error!("Failed to close child window: {e}"));
                        }
                        app_handle.exit(0);
                    }
                }
            });

            let _ = adaptive_sampler::initialize();
            let _ = tray::initialize(app_handle.clone());
            let _ = controller::initialize(app_handle.clone());
            let _ = xeno_utils::initialize();
            let _ = setting::initialize();
            let _ = mapping::initialize();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
