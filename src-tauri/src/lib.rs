// "windows": [
//       {
//         "label": "main",
//         "title": "XialloControl",
//         "fullscreen": false,
//         "decorations": true,
//         "width": 1080,
//         "height": 720,
//         "resizable": false,
//         "transparent": false,
//         "maximizable": false
//       }
//     ],

use crate::xeno_utils::get_config_path;
use once_cell::sync::Lazy;
use std::env;
use std::sync::Mutex;
use tauri::{AppHandle, WebviewWindow, Window};
use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::fern;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use tauri_plugin_updater::{Update, UpdaterExt};
use url::Url;

mod adaptive_sampler;
mod controller;
mod mapping;
mod preset;
mod setting;
mod setup;
mod tray;
mod xeno_utils;

static GITHUB_MIRROR_PREFIX: &str = "https://ghfast.top/";
static UPDATE_CACHE: Lazy<Mutex<Option<Update>>> = Lazy::new(|| Mutex::new(None));

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
    let platform = tauri_plugin_os::platform();
    log::debug!("Platform: {platform}");
    platform.to_string()
}

#[tauri::command]
fn open_devtools(webview: tauri::WebviewWindow) {
    #[cfg(debug_assertions)]
    {
        webview.open_devtools();
    }
}

#[tauri::command]
fn is_release_env() -> bool {
    !cfg!(debug_assertions)
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
    // let decorations = true;
    let decorations = false;

    WebviewWindowBuilder::new(
        &app_handle.clone(),
        "main",
        WebviewUrl::App("index.html".into()),
    )
    .title("XialloControl")
    .min_inner_size(1130.0, 740.0)
    .resizable(true)
    .fullscreen(false)
    .decorations(decorations)
    .transparent(false)
    .maximizable(false)
    .auto_resize()
    .center()
    .enable_clipboard_access()
    .devtools(cfg!(debug_assertions))
    .visible(false)
    .build()
    .expect("Failed to create main window")
}

#[tauri::command]
fn get_locale() -> String {
    let locale = tauri_plugin_os::locale();
    if let Some(locale) = locale {
        log::info!("成功获取到系统语言: {locale:#?}");
        locale
    } else {
        log::warn!("无法获取系统语言，使用默认语言 zh-CN");
        "zh-CN".to_string()
    }
}

#[derive(Clone, serde::Serialize)]
struct UpdateInfo {
    version: String,
    body: String,
    date: String,
}

#[tauri::command]
async fn check_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    log::info!("Checking for updates...");

    let updater = match app.updater_builder().build() {
        Ok(updater) => updater,
        Err(e) => return Err(e.to_string()),
    };

    match updater.check().await {
        Ok(Some(mut update)) => {
            log::info!("Update available: {}", update.version);

            let locale = get_locale();
            if locale.starts_with("zh") {
                let new_url = format!("{}{}", GITHUB_MIRROR_PREFIX, update.download_url);
                log::info!("Using mirror URL for CN user: {new_url}");
                update.download_url = Url::parse(&new_url).map_err(|e| e.to_string())?;
            }

            let info = UpdateInfo {
                version: update.version.clone(),
                body: update.body.clone().unwrap_or_default(),
                date: update.date.map_or("".to_string(), |d| d.date().to_string()),
            };

            *UPDATE_CACHE.lock().unwrap() = Some(update);

            Ok(Some(info))
        }
        Ok(None) => {
            log::info!("Update to date");
            *UPDATE_CACHE.lock().unwrap() = None;
            Ok(None)
        }
        Err(e) => {
            log::error!("Failed to check for updates: {e}");
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn perform_update(app: AppHandle) -> Result<(), String> {
    log::info!("Performing update from cache...");

    let update_to_install = { UPDATE_CACHE.lock().unwrap().take() };

    if let Some(update) = update_to_install {
        log::info!("Update found in cache, starting download...");
        log::info!("Download URL: {}", update.download_url);

        // TODO: 添加下载进度显示
        if let Err(e) = update.download_and_install(|_, _| {}, || {}).await {
            log::error!("Failed to install update: {e}");
            return Err(e.to_string());
        }
        log::info!("Update installed, restarting...");
        app.restart();
    } else {
        log::info!("No update found in cache to perform.");
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            // Write your code here...
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Folder {
                        path: get_config_path("logs"),
                        file_name: None,
                    },
                ))
                .level(log::LevelFilter::Debug)
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .max_file_size(1024 * 128 /* bytes */)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .with_colors(
                    ColoredLevelConfig::new()
                        .error(fern::colors::Color::BrightRed)
                        .warn(fern::colors::Color::Yellow)
                        .info(fern::colors::Color::Green)
                        .debug(fern::colors::Color::Blue)
                        .trace(fern::colors::Color::White),
                )
                .build(),
        )
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            open_devtools,
            is_release_env,
            get_locale,
            check_update,
            perform_update,
            controller::controller::query_devices,
            controller::controller::use_device,
            controller::controller::disconnect_device,
            controller::controller::physical_disconnect_device,
            controller::controller::set_frequency,
            controller::controller::get_controller_data,
            controller::controller::try_auto_connect_last_device,
            controller::calibrate::get_calibration_state,
            controller::calibrate::start_stick_calibration,
            controller::calibrate::next_stick_calibration_step,
            controller::calibrate::cancel_stick_calibration,
            controller::calibrate::save_current_calibration,
            controller::calibrate::reset_calibration_to_default,
            controller::calibrate::set_calibration_mode,
            setting::get_current_settings,
            setting::update_settings,
            mapping::set_mapping,
            mapping::get_mappings,
            mapping::get_mapping_by_id,
            mapping::update_mapping,
            mapping::add_mapping,
            mapping::delete_mapping,
            mapping::refresh_mappings,
            mapping::update_mappings_order,
            preset::load_preset,
            preset::update_deadzone,
            preset::check_presets_list,
            preset::create_preset,
            preset::delete_preset,
            preset::rename_preset,
            preset::switch_to_preset,
            preset::update_preset_items
        ])
        .setup(|app| {
            let app_handle = app.handle();

            // 在创建窗口前，先初始化设置
            setup::initialize();

            let main_window = create_main_window(app_handle.clone());

            // 根据设置判断是否显示窗口
            let settings = setting::get_setting();
            if !settings.minimize_to_tray {
                main_window.show().unwrap();
                main_window.set_focus().unwrap();
            }

            // let child_window = create_child_window(app_handle.clone());
            // let child_windows_list = vec![child_window.clone()];
            let child_windows_list: Vec<WebviewWindow> = vec![];

            main_window.on_window_event({
                let app_handle = app_handle.clone(); // 闭包再 clone 一次，保证 'static
                move |event| match event {
                    tauri::WindowEvent::CloseRequested { .. } => {
                        log::info!("Main window close requested");
                        for w in &child_windows_list {
                            w.close().unwrap_or_else(|e| {
                                log::error!("Failed to close child window: {e}")
                            });
                        }
                        app_handle.exit(0);
                    }
                    // 捕获并忽略 DragDrop 事件，以防止 Tauri 后端劫持前端的拖拽操作
                    tauri::WindowEvent::DragDrop(e) => {
                        // 什么都不做，消费掉这个事件
                        log::debug!("DragDrop event ignored: {e:?}");
                    }
                    _ => {}
                }
            });

            let _ = tray::initialize(app_handle.clone());

            controller::initialize(app_handle.clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
