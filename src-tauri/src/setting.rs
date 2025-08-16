use crate::xeno_utils;
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

const SETTINGS_FILE: &str = "settings.toml";
const DEFAULT_POLLING_FREQUENCY: u32 = 125;
const DEFAULT_DEADZONE: u8 = 10;

pub static GLOBAL_SETTINGS: Lazy<RwLock<AppSettings>> =
    Lazy::new(|| RwLock::new(load_settings_internal()));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    #[serde(default = "bool_false")]
    pub auto_start: bool,

    #[serde(default = "bool_false")]
    pub minimize_to_tray: bool,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_polling_frequency")]
    pub polling_frequency: u32,

    pub previous_preset: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_start: true,
            minimize_to_tray: true,
            theme: "light".to_string(),
            polling_frequency: DEFAULT_POLLING_FREQUENCY,
            previous_preset: "default".to_string(),
        }
    }
}

fn bool_true() -> bool {
    true
}
fn bool_false() -> bool {
    false
}
fn default_theme() -> String {
    "light".to_string()
}
fn default_polling_frequency() -> u32 {
    DEFAULT_POLLING_FREQUENCY
}

/// 获取当前设置
pub fn get_setting() -> AppSettings {
    GLOBAL_SETTINGS.read().unwrap().clone()
}

/// 内部加载设置实现
fn load_settings_internal() -> AppSettings {
    let settings_path = xeno_utils::get_config_path(SETTINGS_FILE);
    xeno_utils::ensure_config_dir();

    if !settings_path.exists() {
        let default_settings = AppSettings::default();
        if let Err(e) = xeno_utils::write_toml_file(&settings_path, &default_settings) {
            log::error!("创建默认设置失败: {e}");
        }
        return default_settings;
    }

    xeno_utils::read_toml_file(&settings_path).unwrap_or_else(|e| {
        log::error!("加载设置失败: {e}, 使用默认设置");
        AppSettings::default()
    })
}

/// 加载应用到全局设置
pub fn load_settings() {
    *GLOBAL_SETTINGS.write().unwrap() = load_settings_internal();
}

/// 保存全局设置到文件
pub async fn save_settings() -> Result<()> {
    let settings = GLOBAL_SETTINGS.read().unwrap().clone();
    let settings_path = xeno_utils::get_config_path(SETTINGS_FILE);
    xeno_utils::ensure_config_dir();

    xeno_utils::write_toml_file(&settings_path, &settings)
        .context("保存设置失败")
        .map_err(|e| {
            log::error!("保存设置失败: {e}");
            e
        })?;

    log::info!("设置已保存到: {:?}", settings_path);
    Ok(())
}

#[tauri::command]
pub async fn update_settings(app: AppHandle, new_settings: AppSettings) -> Result<(), String> {
    log::debug!("接收到更新设置请求: {:?}", new_settings);

    // 1. 验证数据
    if !(1..=8000).contains(&new_settings.polling_frequency) {
        return Err("轮询频率必须在1-8000Hz范围内".to_string());
    }

    // 2. 更新全局缓存
    let mut settings_lock = GLOBAL_SETTINGS.write().unwrap();
    let old_auto_start = settings_lock.auto_start;
    *settings_lock = new_settings.clone();
    drop(settings_lock); // 提前释放锁

    // 3. 异步保存到文件
    tokio::spawn(async move {
        if let Err(e) = save_settings().await {
            log::error!("异步保存设置失败: {:?}", e);
        }
    });

    // 4. 根据新旧数据差异执行其他命令
    if new_settings.auto_start != old_auto_start {
        let autostart_manager = app.autolaunch();
        if new_settings.auto_start {
            if let Err(e) = autostart_manager.enable() {
                log::error!("启用开机自启动失败: {:?}", e);
            }
            log::info!("已启用开机自启动");
        } else {
            if let Err(e) = autostart_manager.disable() {
                log::error!("禁用开机自启动失败: {:?}", e);
            }
            log::info!("已禁用开机自启动");
        }
    }

    log::info!("设置已成功更新");
    Ok(())
}

#[tauri::command]
pub async fn get_current_settings() -> Result<AppSettings, String> {
    log::debug!("前端请求当前设置");
    Ok(get_setting())
}

pub fn initialize() {
    log::debug!("初始化设置");
    load_settings();
}
