use tauri::Manager;
use tauri::Window;

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
    window.minimize().map_err(|e| format!("最小化窗口失败: {}", e))
}

mod controller;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            close_current_window,
            minimize_current_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
