use crate::xeno_utils::get_app_root;
// ---------------------- 外部依赖 ----------------------
use hidapi::{HidApi};
use once_cell::sync::Lazy;
use rusty_xinput::{XInputHandle};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::{fs, thread, time::Duration};
use tauri::{AppHandle, Emitter};
// ---------------------- 设备信息结构体 ----------------------

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

fn poll_xbox(device: &DeviceInfo) {
    // TODO: 调用你的 xbox 轮询函数
    let xinput = get_xinput();
    let compose_code: u32 = 0x00;

    match xinput.get_state(0) {
        Ok(state) => {
            // 象征性使用 Rust 风格的方法判断按钮
            if state.south_button() {
                println!("Xbox A 键（South）被按下");
            }
            if state.east_button() {
                println!("Xbox B 键（East）被按下");
            }
            if state.north_button() {
                println!("Xbox Y 键（North）被按下");
            }
            if state.west_button() {
                println!("Xbox X 键（West）被按下");
            }

            // 摇杆坐标
            let (lx, ly) = state.left_stick_normalized();
            println!("左摇杆 raw = ({}, {})", lx, ly);
            // let (rx, ry) = state.right_stick_raw();
            // println!("右摇杆 raw = ({}, {})", rx, ry);
        }
        Err(err) => {
            println!("手柄未连接或无法读取状态: {:?}", err);
            // TODO: 处理异常情况
            disconnect_device();
            let app_handle = get_app_handle();
            if let Err(e) = app_handle.emit("physical_connect_status", false) {
                log::error!("发送 physical_connect_status 事件失败: {}", e);
            }
        }
    }
}

/// 轮询设备异步函数
 fn poll_controller(device: &DeviceInfo) {
    match device.controller_type {
        ControllerType::Xbox => {
            // log::debug!("轮询 Xbox 设备: {}", device.name);
            // TODO: 调用你的 xbox 轮询函数
            poll_xbox(device);
        }
        _ => {
            // log::debug!("轮询其他设备: {}", device.name);
            // TODO: 调用其他设备轮询函数
            // 例如 poll_other(device, app_handle).await;
        }
    }
}

pub fn listen() {
    thread::spawn( ||  {
        log::info!("🎧 启动设备监听任务");

        let mut last_device: Option<DeviceInfo> = None;

        loop {
            let time_interval = *TIME_INTERVAL.lock().unwrap();
            let current_device = CURRENT_DEVICE.lock().unwrap().clone();

            let is_current_valid = current_device.device_path.is_some();
            let is_last_valid = last_device
                .as_ref()
                .map(|d| d.device_path.is_some())
                .unwrap_or(false);

            // 设备连接/切换/断开检测
            match (is_last_valid, is_current_valid) {
                (false, true) => {
                    log::info!("🔌 连接新设备: {}", current_device.name);
                    last_device = Some(current_device.clone());
                    // TODO: 初始化监听逻辑
                }
                (true, true) => {
                    if last_device.as_ref().unwrap().device_path != current_device.device_path {
                        log::info!(
                            "🔄 设备切换: {} → {}",
                            last_device.as_ref().unwrap().name,
                            current_device.name
                        );
                        last_device = Some(current_device.clone());
                        // TODO: 切换监听逻辑
                    }
                    // 设备相同，不操作
                }
                (true, false) => {
                    log::info!("❌ 设备断开: {}", last_device.as_ref().unwrap().name);
                    last_device = None;
                    // TODO: 清理监听逻辑
                }
                (false, false) => {
                    // 无设备，不操作
                }
            }

            // 调用轮询函数
            if let Some(device) = &last_device {
                poll_controller(device);
            }

            thread::sleep(Duration::from_secs_f32(time_interval));
        }
    });
}

pub fn initialize(app_handle: AppHandle) {
    let xinput = XInputHandle::load_default().unwrap();
    let mut handles = HANDLES.lock().unwrap();

    *handles = Some(Handles {
        app_handle: app_handle.clone(),
        xinput,
    });

    _list_supported_devices();

    polling_devices();
    listen();
}
