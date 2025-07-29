use crate::xeno_utils::get_app_root;
// ---------------------- 外部依赖 ----------------------
use crate::adaptive_sampler::AdaptiveSampler;
use gilrs::{Button, Event, EventType, GamepadId, Gilrs};
use hidapi::HidApi;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
#[cfg(target_os = "windows")]
use rusty_xinput::{XInputHandle, XInputState};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Mutex, OnceLock, RwLock};
use std::{fs, thread, time::Duration};
use tauri::{AppHandle, Emitter};

// ---------------------- 常量定义 ----------------------
pub static SUPPORTED_DEVICES_FILE: &str = "supported_devices.toml";
pub static FREQ: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(125));
pub static SAMPLING_RATE: Lazy<Mutex<f64>> = Lazy::new(|| Mutex::new(1000.0));
pub static TIME_INTERVAL: Lazy<Mutex<f32>> = Lazy::new(|| Mutex::new(1.0));

// ---------------------- 结构体定义 ----------------------
/// 设备信息结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub vendor_id: String,
    pub product_id: Option<String>,    // 配置时可选，运行时检测设备时一般有值
    pub device_path: Option<String>,   // 连接设备专属，配置时为 None
    pub controller_type: ControllerType, // 设备类型
}

/// 全局句柄存储结构
pub struct Handles {
    pub app_handle: AppHandle,
    #[cfg(target_os = "windows")]
    pub xinput_handle: XInputHandle,
}

/// TOML配置结构
#[derive(Debug, Serialize, Deserialize)]
struct SupportedDevicesConfig {
    devices: Vec<DeviceInfo>,
}

// ---------------------- 枚举定义 ----------------------
/// 控制器类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerType {
    Xbox,
    PlayStation,
    Switch,
    BETOP,
    Other,
}

// ---------------------- 全局静态变量 ----------------------
static HANDLES: Lazy<Mutex<Option<Handles>>> = Lazy::new(|| Mutex::new(None));
pub static CURRENT_DEVICE: Lazy<Mutex<DeviceInfo>> = Lazy::new(|| {
    Mutex::new(DeviceInfo {
        name: "".into(),
        vendor_id: "".into(),
        product_id: None,
        device_path: None,
        controller_type: ControllerType::Other,
    })
});
pub static ADAPTER: Lazy<Mutex<AdaptiveSampler>> = Lazy::new(|| {
    Mutex::new(AdaptiveSampler::new(200_000.0, 10.0))
});
pub static GILRS_TX: OnceLock<Sender<(GamepadId, EventType)>> = OnceLock::new();
pub static GILRS_RX: OnceLock<Mutex<Receiver<(GamepadId, EventType)>>> = OnceLock::new();
pub static GLOBAL_GILRS: Lazy<Mutex<Option<Gilrs>>> = Lazy::new(|| Mutex::new(None));
static LATEST_EVENT_TYPE: OnceLock<RwLock<Option<EventType>>> = OnceLock::new();

// ---------------------- 控制器类型检测 ----------------------
/// 根据厂商ID判断控制器类型
pub fn detect_controller_type(vid: &str) -> ControllerType {
    match vid.to_ascii_lowercase().as_str() {
        "045e" => ControllerType::Xbox,
        "054c" => ControllerType::PlayStation,
        "057e" => ControllerType::Switch,
        "20bc" => ControllerType::BETOP,
        _ => ControllerType::Other,
    }
}

// ---------------------- 配置管理 ----------------------
/// 默认支持的设备列表（配置用）
fn default_devices() -> Vec<DeviceInfo> {
    vec![
        DeviceInfo {
            name: "Any Xbox Controller".into(),
            vendor_id: "045e".into(),
            product_id: None,
            device_path: None,
            controller_type: ControllerType::Xbox,
        },
        DeviceInfo {
            name: "DualShock 4 (PS4)".into(),
            vendor_id: "054c".into(),
            product_id: None,
            device_path: None,
            controller_type: ControllerType::PlayStation,
        },
        DeviceInfo {
            name: "DualSense (PS5)".into(),
            vendor_id: "054c".into(),
            product_id: None,
            device_path: None,
            controller_type: ControllerType::PlayStation,
        },
        DeviceInfo {
            name: "Switch Pro".into(),
            vendor_id: "057e".into(),
            product_id: None,
            device_path: None,
            controller_type: ControllerType::Switch,
        },
        DeviceInfo {
            name: "[ BETOP CONTROLLER ]".into(),
            vendor_id: "20bc".into(),
            product_id: Some("1263".into()),
            device_path: None,
            controller_type: ControllerType::Other,
        },
    ]
}

/// 从配置文件加载支持的设备，如果不存在则生成默认配置文件
pub fn load_or_create_config(path: &str) -> Vec<DeviceInfo> {
    let config_path = get_app_root().join(path);

    if config_path.exists() {
        let toml_str = match fs::read_to_string(config_path) {
            Ok(content) => content,
            Err(e) => {
                log::error!("读取 TOML 配置文件失败: {}", e);
                return default_devices();
            }
        };

        match toml::from_str::<SupportedDevicesConfig>(&toml_str) {
            Ok(mut config) => {
                // 确保配置中的每个设备都有正确的controller_type（兼容旧配置）
                for device in &mut config.devices {
                    device.controller_type = detect_controller_type(&device.vendor_id);
                }
                config.devices
            }
            Err(e) => {
                log::error!("解析 TOML 配置文件失败: {}", e);
                default_devices()
            }
        }
    } else {
        println!("🛠️ 配置文件不存在，正在生成默认 TOML 配置...");
        println!("{:?}", config_path);

        let default = default_devices();
        let config = SupportedDevicesConfig {
            devices: default.clone(),
        };

        match toml::to_string_pretty(&config) {
            Ok(toml_str) => {
                if let Err(e) = fs::write(config_path, toml_str) {
                    log::error!("写入默认 TOML 配置文件失败: {}", e);
                }
            }
            Err(e) => log::error!("序列化 TOML 配置文件失败: {}", e),
        }

        default
    }
}

// ---------------------- 设备检测 ----------------------
/// 根据配置过滤当前连接的支持设备，补充运行时设备信息
pub fn list_supported_connected_devices(config: &[DeviceInfo]) -> Vec<DeviceInfo> {
    let api = match HidApi::new() {
        Ok(api) => api,
        Err(e) => {
            log::error!("初始化 hidapi 失败: {}", e);
            return Vec::new();
        }
    };

    let mut supported_devices = Vec::new();

    for device in api.device_list() {
        let vid = format!("{:04x}", device.vendor_id());
        let pid = format!("{:04x}", device.product_id());

        // 匹配配置支持的设备（厂商ID和可选产品ID匹配）
        let matched = config.iter().find(|d| {
            d.vendor_id.eq_ignore_ascii_case(&vid)
                && match &d.product_id {
                Some(pid_cfg) => pid_cfg.eq_ignore_ascii_case(&pid),
                None => true,
            }
        });

        if let Some(supported) = matched {
            let device_info = DeviceInfo {
                name: device.product_string().unwrap_or("未知设备").to_string(),
                vendor_id: vid.clone(),
                product_id: Some(pid.clone()),
                device_path: Some(device.path().to_string_lossy().to_string()),
                controller_type: detect_controller_type(&vid),
            };
            supported_devices.push(device_info);
        }
    }

    supported_devices
}

// ---------------------- 工具函数 ----------------------
pub fn get_app_handle() -> AppHandle {
    HANDLES
        .lock()
        .unwrap()
        .as_ref()
        .expect("HANDLES not initialized")
        .app_handle
        .clone()
}

#[cfg(target_os = "windows")]
pub fn get_xinput() -> XInputHandle {
    HANDLES
        .lock()
        .unwrap()
        .as_ref()
        .expect("HANDLES not initialized")
        .xinput_handle
        .clone()
}

fn _list_supported_devices() -> Vec<DeviceInfo> {
    let config = load_or_create_config(SUPPORTED_DEVICES_FILE);
    list_supported_connected_devices(&config)
}

fn _query_devices() -> Vec<DeviceInfo> {
    _list_supported_devices()
}

fn _find_device_by_name(name: &str) -> Option<DeviceInfo> {
    _list_supported_devices()
        .into_iter()
        .find(|d| d.name == name)
}

// ---------------------- Tauri 命令接口 ----------------------
#[tauri::command]
pub async fn query_devices(app: AppHandle) -> Vec<DeviceInfo> {
    let devices = _query_devices();
    if let Err(e) = app.emit("update_devices", devices.clone()) {
        log::error!("发送 update_devices 事件失败: {}", e);
    }
    log::debug!("执行了 query_devices 命令");
    devices
}

#[tauri::command]
pub async fn use_device(device_name: String) -> bool {
    log::debug!("尝试使用设备: {}", device_name);
    match _find_device_by_name(&device_name) {
        Some(device) => {
            let mut current_device = CURRENT_DEVICE.lock().unwrap();
            *current_device = device;
            log::info!("✅ 使用设备: {}", current_device.name);
            true
        }
        None => {
            log::error!("❌ 未找到名为 '{}' 的设备", device_name);
            false
        }
    }
}

#[tauri::command]
pub fn disconnect_device() -> bool {
    log::debug!("尝试断开设备连接");
    let mut current_device = CURRENT_DEVICE.lock().unwrap();
    *current_device = default_devices()[0].clone();
    log::info!("✅ 已断开当前设备");
    true
}

#[tauri::command]
pub async fn set_frequency(freq: u32) {
    let freq = freq.clamp(1, 8000);
    let mut global_freq = FREQ.lock().unwrap();
    let mut time_interval = TIME_INTERVAL.lock().unwrap();
    let mut sample_rate = SAMPLING_RATE.lock().unwrap();
    let adapter = ADAPTER.lock().unwrap();

    *global_freq = freq;
    *sample_rate = adapter.compute_sampling_rate(freq as f64);
    *time_interval = 1.0 / freq as f32;

    log::info!(
        "轮询频率: {} Hz ({}秒), 采样率: {:.2} Hz",
        *global_freq, *time_interval, *sample_rate
    );
}

// ---------------------- 设备轮询 ----------------------
fn poll_other_controllers(device: &DeviceInfo) {
    let gilrs_guard = GLOBAL_GILRS.lock().unwrap();
    let gilrs = gilrs_guard.as_ref().unwrap();

    for (_id, gamepad) in gilrs.gamepads() {
        let vid = format!("{:04x}", gamepad.vendor_id().unwrap());
        let pid = format!("{:04x}", gamepad.product_id().unwrap());

        if vid.eq_ignore_ascii_case(&device.vendor_id)
            && pid.eq_ignore_ascii_case(&device.product_id.as_deref().unwrap())
        {
            if gamepad.is_pressed(Button::South) {
                println!("----------------- Button::South 键被按下");
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn _poll_xbox_controller_state(state: XInputState) {
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
    if state.guide_button() {
        println!("Xbox Guide 键被按下");
    }
    if state.start_button() {
        println!("Xbox Start 键被按下");
    }
    if state.left_thumb_button() {
        println!("Xbox 左摇杆按下");
    }
    if state.right_thumb_button() {
        println!("Xbox 右摇杆按下");
    }

    let (lx, ly) = state.left_stick_normalized();
    println!("左摇杆 raw = ({}, {})", lx, ly);
}

#[cfg(target_os = "windows")]
fn poll_xbox_controller(device: &DeviceInfo) {
    let xinput = get_xinput();
    match xinput.get_state_ex(0).or_else(|_| xinput.get_state(0)) {
        Ok(state) => _poll_xbox_controller_state(state),
        Err(_) => {
            disconnect_device();
            let app_handle = get_app_handle();
            if let Err(e) = app_handle.emit("physical_connect_status", false) {
                log::error!("发送 physical_connect_status 事件失败: {}", e);
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn poll_xbox_controller(_device: &DeviceInfo) {
    println!("poll_xbox_controllers");
}

/// 根据设备类型执行对应的轮询操作
fn poll_controller(device: &DeviceInfo) {
    match device.controller_type {
        ControllerType::Xbox => poll_xbox_controller(device),
        _ => poll_other_controllers(device),
    }
}

// ---------------------- 后台任务 ----------------------
/// 后台设备发现任务（500ms间隔）
pub fn polling_devices() {
    tauri::async_runtime::spawn(async move {
        log::info!("🛠️ 控制器监听已启动...");
        let app_handle = get_app_handle();
        loop {
            let devices = _query_devices();
            if let Err(e) = app_handle.emit("update_devices", devices.clone()) {
                log::error!("发送 update_devices 事件失败: {}", e);
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
}

/// 主设备监听循环
/// 主设备监听循环
pub fn listen() {
    thread::spawn(|| {
        log::info!("🎧 启动设备监听任务");
        let mut last_device: Option<DeviceInfo> = None;

        loop {
            let time_interval = *TIME_INTERVAL.lock().unwrap();
            let current_device = CURRENT_DEVICE.lock().unwrap().clone();

            // 检查设备连接状态变化 - 修复类型匹配问题
            let last_has_device = last_device.is_some();
            let current_has_device = current_device.device_path.is_some();

            match (last_has_device, current_has_device) {
                (false, true) => {
                    log::info!("🔌 连接新设备: {}", current_device.name);
                    last_device = Some(current_device.clone());
                }
                (true, true) if last_device.as_ref().unwrap().device_path != current_device.device_path => {
                    log::info!(
                        "🔄 设备切换: {} → {}",
                        last_device.as_ref().unwrap().name,
                        current_device.name
                    );
                    last_device = Some(current_device.clone());
                }
                (true, false) => {
                    if let Some(device) = &last_device {
                        log::info!("❌ 设备断开: {}", device.name);
                    }
                    last_device = None;
                }
                _ => (), // 无状态变化
            }

            // 执行设备轮询
            if let Some(device) = &last_device {
                poll_controller(device);
            }

            thread::sleep(Duration::from_secs_f32(time_interval));
        }
    });
}

/// 初始化Gilrs事件监听线程
pub fn gilrs_listen() {
    std::thread::spawn(move || {
        let gilrs = Gilrs::new().expect("Failed to init Gilrs");
        {
            *GLOBAL_GILRS.lock().unwrap() = Some(gilrs);
        }

        loop {
            if let Some(gilrs) = GLOBAL_GILRS.lock().unwrap().as_mut() {
                while let Some(Event { event, .. }) = gilrs.next_event() {
                    // 事件处理逻辑（当前仅清空事件队列）
                }
            }
            std::thread::sleep(Duration::from_secs_f32(
                1.0 / *SAMPLING_RATE.lock().unwrap() as f32
            ));
        }
    });
}

// ---------------------- 初始化函数 ----------------------
/// 初始化全局句柄
fn query_needed_handle(app_handle: AppHandle) {
    *HANDLES.lock().unwrap() = Some(Handles {
        app_handle: app_handle.clone(),
        #[cfg(target_os = "windows")]
        xinput_handle: XInputHandle::load_default().unwrap(),
    });
}

/// 模块初始化入口
pub fn initialize(app_handle: AppHandle) {
    query_needed_handle(app_handle);
    gilrs_listen();
    polling_devices();
    listen();
}