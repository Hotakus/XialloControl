use hidapi::HidApi;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

// 修改为 TOML 配置文件
pub static SUPPORTED_DEVICES_FILE: &str = "supported_devices.toml";

// 添加 Clone trait 实现
#[derive(Debug, Serialize, Deserialize, Clone)] // 添加 Clone trait
pub struct SupportedDevice {
    pub name: String,
    pub vendor_id: String,
    pub product_id: Option<String>, // 可选字段
}

pub struct SupportedConnectedDevice {
    pub name: String,
    pub vendor_id: String,
    pub product_id: String,
    pub device_path: String, // 唯一标识，可用来打开设备
}

// 包装结构体用于 TOML 序列化
#[derive(Debug, Serialize, Deserialize)]
struct SupportedDevicesConfig {
    devices: Vec<SupportedDevice>,
}

fn default_devices() -> Vec<SupportedDevice> {
    vec![
        SupportedDevice {
            name: "Any Xbox Controller".into(),
            vendor_id: "045e".into(),
            product_id: None,
        },
        SupportedDevice {
            name: "DualShock 4 (PS4)".into(),
            vendor_id: "054c".into(),
            product_id: None,
        },
        SupportedDevice {
            name: "DualSense (PS5)".into(),
            vendor_id: "054c".into(),
            product_id: None,
        },
    ]
}

pub fn load_or_create_config(path: &str) -> Vec<SupportedDevice> {
    let config_path = Path::new(path);

    if config_path.exists() {
        // 读取 TOML 文件
        let toml_str = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                log::error!("Failed to read TOML config: {}", e);
                return default_devices();
            }
        };

        // 解析 TOML
        match toml::from_str::<SupportedDevicesConfig>(&toml_str) {
            Ok(config) => config.devices,
            Err(e) => {
                log::error!("Failed to parse TOML config: {}", e);
                default_devices()
            }
        }
    } else {
        println!("🛠️ Config not found. Generating default TOML config...");

        let default = default_devices();
        let config = SupportedDevicesConfig {
            devices: default.clone(),
        };

        // 序列化为 TOML
        match toml::to_string_pretty(&config) {
            Ok(toml_str) => {
                if let Err(e) = fs::write(path, toml_str) {
                    log::error!("Failed to write default TOML config: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to serialize TOML config: {}", e);
            }
        }

        default
    }
}

// 以下函数保持不变
pub fn list_supported_connected_devices(
    config: &[SupportedDevice],
) -> Vec<SupportedConnectedDevice> {
    let api = match HidApi::new() {
        Ok(api) => api,
        Err(e) => {
            log::error!("Failed to init hidapi: {}", e);
            return Vec::new();
        }
    };

    let mut supported_devices = Vec::new();

    for device in api.device_list() {
        let vid = format!("{:04x}", device.vendor_id());
        let pid = format!("{:04x}", device.product_id());

        let matched = config.iter().find(|d| {
            d.vendor_id == vid
                && match &d.product_id {
                    Some(pid_cfg) => pid_cfg == &pid,
                    None => true,
                }
        });

        if let Some(_supported) = matched {
            let device_info = SupportedConnectedDevice {
                name: device
                    .product_string()
                    .unwrap_or("Unknown Device")
                    .to_string(),
                vendor_id: vid.clone(),
                product_id: pid.clone(),
                device_path: device.path().to_string_lossy().to_string(),
            };
            supported_devices.push(device_info);
        }
    }
    supported_devices
}

fn _query_devices() -> Vec<String> {
    let config = load_or_create_config(SUPPORTED_DEVICES_FILE);
    let devices = list_supported_connected_devices(&config);

    devices.iter().map(|device| device.name.clone()).collect()
}

#[tauri::command]
pub async fn query_devices(app: tauri::AppHandle) -> Vec<String> {
    let devices_name = _query_devices();
    if let Err(e) = app.emit("update_devices", devices_name.clone()) {
        log::error!("Failed to emit update_devices event: {}", e);
    }
    log::debug!("query_devices");
    devices_name
}

pub fn listen(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        log::info!("🛠️ Controller listening...");

        loop {
            let devices_name = _query_devices();
            if let Err(e) = app_handle.emit("update_devices", devices_name.clone()) {
                log::error!("Failed to emit update_devices event: {}", e);
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
}
