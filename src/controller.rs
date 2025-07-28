use crate::xeno_utils::get_app_root;
// ---------------------- 外部依赖 ----------------------
use hidapi::HidApi;
use once_cell::sync::Lazy;
use gilrs::Gilrs;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::{fs, thread, time::Duration};
use tauri::{AppHandle, Emitter};

#[cfg(target_os = "windows")]
use rusty_xinput::{XInputHandle, XInputState};
// ---------------------- 设备信息结构体 ----------------------

/// 设备信息，既可表示支持的设备配置，也可表示已连接设备
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub vendor_id: String,
    pub product_id: Option<String>, // 配置时可选，运行时检测设备时一般有值
    pub device_path: Option<String>, // 连接设备专属，配置时为 None
    pub controller_type: ControllerType, // 设备类型
}

// ---------------------- 常量定义 ----------------------

pub struct Handles {
    pub app_handle: AppHandle,

    pub gilrs_handle: Gilrs,

    #[cfg(target_os = "windows")]
    pub xinput_handle: XInputHandle,
}

static HANDLES: Lazy<Mutex<Option<Handles>>> = Lazy::new(|| Mutex::new(None));

pub static SUPPORTED_DEVICES_FILE: &str = "supported_devices.toml";
pub static FREQ: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(125));
pub static TIME_INTERVAL: Lazy<Mutex<f32>> = Lazy::new(|| Mutex::new(1.0));
pub static CURRENT_DEVICE: Lazy<Mutex<DeviceInfo>> = Lazy::new(|| {
    Mutex::new(DeviceInfo {
        name: "".into(),
        vendor_id: "".into(),
        product_id: None,
        device_path: None,
        controller_type: ControllerType::Other,
    })
});

// ---------------------- 控制器类型定义 ----------------------

/// 控制器类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerType {
    Xbox,
    PlayStation,
    Switch,
    Other,
}

/// 根据厂商ID判断控制器类型
pub fn detect_controller_type(vid: &str) -> ControllerType {
    match vid.to_ascii_lowercase().as_str() {
        "045e" => ControllerType::Xbox,
        "054c" => ControllerType::PlayStation,
        "057e" => ControllerType::Switch,
        _ => ControllerType::Other,
    }
}

// ---------------------- TOML 配置结构 ----------------------

#[derive(Debug, Serialize, Deserialize)]
struct SupportedDevicesConfig {
    devices: Vec<DeviceInfo>,
}

// ---------------------- 默认设备配置 ----------------------

/// 默认支持的设备列表（配置用）
fn default_devices() -> Vec<DeviceInfo> {
    let devices = vec![
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
    ];

    devices
}

// ---------------------- 配置加载 ----------------------

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
                // 确保配置中的每个设备都有正确的 controller_type（兼容旧配置）
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

        let mut default = default_devices();
        // 默认设备的 controller_type 已设置

        let config = SupportedDevicesConfig {
            devices: default.clone(),
        };

        match toml::to_string_pretty(&config) {
            Ok(toml_str) => {
                if let Err(e) = fs::write(config_path, toml_str) {
                    log::error!("写入默认 TOML 配置文件失败: {}", e);
                }
            }
            Err(e) => {
                log::error!("序列化 TOML 配置文件失败: {}", e);
            }
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
            // println!(
            //     "---------\n\n发现设备: {:?} \
            //  \n厂商ID: {:?} \
            //  \n产品ID: {:?} \
            //  \n厂商  ：{:?} \
            //  \n序列号：{:?} \
            //  \n发布号：{:?} \
            //  \nTypeID: {:?} \
            //  \n路径  : {:?} \
            //  \n总线  ：{:?} \
            //  \n用法  ：{:?} \
            //  \n用法页：{:?} \
            //  \n接口  ：{:?}",
            //     device.product_string().unwrap_or("未知设备"),
            //     vid,
            //     pid,
            //     device.manufacturer_string().unwrap_or("未知厂商"),
            //     device.serial_number().unwrap_or("未知序列号"),
            //     device.release_number(),
            //     device.type_id(),
            //     device.path().to_string_lossy().to_string(),
            //     device.bus_type(),
            //     device.usage(),
            //     device.usage_page(),
            //     device.interface_number()
            // );

            // 构造运行时设备信息，带 device_path 和具体 product_id，类型也重新确认
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

// ---------------------- 内部工具函数 ----------------------

pub fn get_app_handle() -> AppHandle {
    let handles = HANDLES.lock().unwrap();
    handles
        .as_ref()
        .expect("HANDLES not initialized")
        .app_handle
        .clone()
}

#[cfg(target_os = "windows")]
pub fn get_xinput() -> XInputHandle {
    let handles = HANDLES.lock().unwrap();
    handles
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
    // devices.iter().map(|d| d.name.clone()).collect()
}

fn _find_device_by_name(name: &str) -> Option<DeviceInfo> {
    let devices = _list_supported_devices();
    devices.into_iter().find(|d| d.name == name)
}

// ---------------------- Tauri 命令接口 ----------------------

#[tauri::command]
pub async fn query_devices(app: AppHandle) -> Vec<DeviceInfo> {
    let devices = _query_devices();
    if let Err(e) = app.emit("update_devices", devices.clone()) {
        log::error!("发送 update_devices 事件失败: {}", e);
    }
    log::debug!("执行了 query_devices 命令");
    log::debug!("设备列表: {:?}", &devices);
    devices
}

#[tauri::command]
pub async fn use_device(device_name: String) -> bool {
    log::debug!("尝试使用设备: {}", device_name);
    match _find_device_by_name(&device_name) {
        Some(device) => {
            log::debug!(
                "找到设备: {}，厂商ID: {}, 产品ID: {}, 设备路径: {:?}, 类型: {:?}",
                device.name,
                device.vendor_id,
                device.product_id.clone().unwrap_or_default(),
                device.device_path.as_deref(),
                device.controller_type
            );

            let mut current_device = CURRENT_DEVICE.lock().unwrap();
            *current_device = device.clone();

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
    let freq = freq.clamp(1, 8000); // 限制范围
    let mut global_freq = FREQ.lock().unwrap();
    let mut time_interval = TIME_INTERVAL.lock().unwrap();

    *global_freq = freq;
    *time_interval = 1.0 / freq as f32;

    log::info!(
        "轮询频率已设置为: {} Hz ({} seconds)",
        *global_freq,
        *time_interval
    );
}

// ---------------------- 后台监听任务 ----------------------

pub fn polling_devices() {
    tauri::async_runtime::spawn(async move {
        log::info!("🛠️ 控制器监听已启动...");
        let app_handle = get_app_handle();
        loop {
            let devices_name = _query_devices();
            if let Err(e) = app_handle.emit("update_devices", devices_name.clone()) {
                log::error!("发送 update_devices 事件失败: {}", e);
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
}


fn poll_other_controllers(device: &DeviceInfo) {
    println!("poll_other_controllers");
}

fn _poll_xbox_controller_state(state: XInputState) {
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

    // 摇杆坐标
    let (lx, ly) = state.left_stick_normalized();
    println!("左摇杆 raw = ({}, {})", lx, ly);
    // let (rx, ry) = state.right_stick_raw();
    // println!("右摇杆 raw = ({}, {})", rx, ry);
}

#[cfg(target_os = "windows")]
fn poll_xbox_controller(device: &DeviceInfo) {
    // TODO: 调用你的 xbox 轮询函数
    let xinput = get_xinput();
    let compose_code: u32 = 0x00;

    match xinput.get_state_ex(0) {
        Ok(ex_state) => {
            _poll_xbox_controller_state(ex_state);
        }
        Err(err) => {
            match xinput.get_state(0) {
                Ok(state) => {
                    _poll_xbox_controller_state(state);
                }
                Err(_) => {
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
    }
}

#[cfg(target_os = "linux")]
fn poll_xbox_controller(device: &DeviceInfo) {
    println!("poll_xbox_controllers");
}


/// 轮询设备异步函数
fn poll_controller(device: &DeviceInfo) {
    match device.controller_type {
        ControllerType::Xbox => {
            // log::debug!("轮询 Xbox 设备: {}", device.name);
            poll_xbox_controller(device);
        }
        _ => {
            // log::debug!("轮询其他设备: {}", device.name);
            // TODO: 调用其他设备轮询函数
            poll_other_controllers(device);
        }
    }
}

pub fn listen() {
    thread::spawn(|| {
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

fn query_needed_handle(app_handle: AppHandle) {
    let mut handles = HANDLES.lock().unwrap();
    *handles = Some(Handles {
        app_handle: app_handle.clone(),
        gilrs_handle: Gilrs::new().unwrap(),

        #[cfg(target_os = "windows")]
        xinput_handle: XInputHandle::load_default().unwrap(),
    });
}

pub fn initialize(app_handle: AppHandle) {
    query_needed_handle(app_handle);
    polling_devices();
    listen();
}
