#![allow(dead_code)]

use crate::xeno_utils;
use crate::xeno_utils::get_app_root;
use anyhow::{Context, Result};
use log;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

// 设置文件名
const SETTINGS_FILE: &str = "settings.toml";

// 默认设置值
const DEFAULT_POLLING_FREQUENCY: u32 = 125; // 125 Hz
const DEFAULT_DEADZONE: u8 = 10; // 10%

/// 应用程序设置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    /// 开机自启动
    #[serde(default = "bool_true")]
    pub auto_start: bool,

    /// 最小化到托盘
    #[serde(default = "bool_true")]
    pub minimize_to_tray: bool,

    /// 界面主题
    #[serde(default = "default_theme")]
    pub theme: String,

    /// 轮询频率 (Hz)
    #[serde(default = "default_polling_frequency")]
    pub polling_frequency: u32,

    /// 右摇杆死区范围 (%)
    #[serde(default = "default_deadzone")]
    pub deadzone: u8,

    /// 左摇杆死区范围 (%)
    #[serde(default = "default_deadzone")]
    pub deadzone_left: u8,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_start: true,
            minimize_to_tray: true,
            theme: "light".to_string(),
            polling_frequency: DEFAULT_POLLING_FREQUENCY,
            deadzone: DEFAULT_DEADZONE,
            deadzone_left: DEFAULT_DEADZONE,
        }
    }
}

// 默认值辅助函数
fn bool_true() -> bool {
    true
}
fn default_theme() -> String {
    "light".to_string()
}
fn default_polling_frequency() -> u32 {
    DEFAULT_POLLING_FREQUENCY
}
fn default_deadzone() -> u8 {
    DEFAULT_DEADZONE
}

/// 获取设置文件路径
fn get_settings_path() -> &'static str {
    SETTINGS_FILE
}

/// 加载应用设置
pub fn load_settings() -> AppSettings {
    let settings_path = xeno_utils::get_config_path(SETTINGS_FILE);
    xeno_utils::ensure_config_dir();

    if !settings_path.exists() {
        save_settings(&AppSettings::default()).unwrap();
        return AppSettings::default();
    }

    xeno_utils::read_toml_file(&settings_path).unwrap_or_else(|e| {
        log::error!("加载设置失败: {}, 使用默认设置", e);
        AppSettings::default()
    })
}

/// 保存应用设置
pub fn save_settings(settings: &AppSettings) -> Result<()> {
    let settings_path = xeno_utils::get_config_path(SETTINGS_FILE);
    xeno_utils::ensure_config_dir();

    xeno_utils::write_toml_file(&settings_path, settings)
        .context("保存设置失败")
        .map_err(|e| {
            log::error!("保存设置失败: {}", e);
            e
        })?;

    log::info!("设置已保存到: {:?}", settings_path);
    Ok(())
}

/// 更新应用设置
#[tauri::command]
pub async fn update_settings(app: AppHandle, new_settings: AppSettings) -> Result<(), String> {
    log::debug!("接收到更新设置请求: {:?}", new_settings);

    // 验证轮询频率范围
    if new_settings.polling_frequency < 1 || new_settings.polling_frequency > 8000 {
        let msg = "轮询频率必须在1-8000Hz范围内".to_string();
        log::error!("{}", msg);
        return Err(msg);
    }

    // 验证死区范围
    if new_settings.deadzone > 30 {
        let msg = "死区范围不能超过30%".to_string();
        log::error!("{}", msg);
        return Err(msg);
    }

    // 保存设置
    save_settings(&new_settings).map_err(|e| {
        let msg = format!("保存设置失败: {}", e);
        log::error!("{}", msg);
        msg
    })?;

    // 获取自动启动管理器
    let autostart_manager = app.autolaunch();
    if new_settings.auto_start && !autostart_manager.is_enabled().unwrap() {
        // 启用 autostart
        let _ = autostart_manager.enable();
        log::info!("已启用开机自启动");
    } else if !new_settings.auto_start && autostart_manager.is_enabled().unwrap() {
        // 禁用 autostart
        let _ = autostart_manager.disable();
        log::info!("已禁用开机自启动");
    }
    log::debug!(
        "registered for autostart? {}",
        autostart_manager.is_enabled().unwrap()
    );

    log::info!("设置已成功更新");
    Ok(())
}

/// 获取当前设置
#[tauri::command]
pub async fn get_current_settings() -> Result<AppSettings, String> {
    log::debug!("获取当前设置");
    let settings = load_settings();
    log::debug!("当前设置: {:?}", settings);
    Ok(settings)
}

pub fn initialize() {
    log::debug!("初始化设置");
}
