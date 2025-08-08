#![allow(dead_code)]

// ---------------------- 外部依赖 ----------------------
use crate::adaptive_sampler::AdaptiveSampler;
use crate::controller::datas::{ControllerButtons, ControllerDatas};
use crate::{mapping, xeno_utils};
use gilrs::{Axis, Button, Event, EventType, Gamepad, GamepadId, Gilrs};
use hidapi::HidApi;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, RwLock};
use std::{thread, time::Duration};
use tauri::{AppHandle, Emitter};

use crate::controller::xbox;
#[cfg(target_os = "windows")]
use rusty_xinput::XInputHandle;

// ---------------------- 结构体定义 ----------------------
/// 游戏控制器设备信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    /// 设备显示名称
    pub name: String,
    /// 厂商ID (16进制字符串)
    pub vendor_id: String,
    /// 产品ID (16进制字符串，可选)
    pub product_id: Option<String>,
    /// 设备路径 (运行时检测)
    pub device_path: Option<String>,
    /// 控制器类型分类
    pub controller_type: ControllerType,
}

/// 全局应用句柄容器
pub struct Handles {
    /// Tauri 应用句柄
    pub app_handle: AppHandle,
    /// Windows XInput 句柄
    #[cfg(target_os = "windows")]
    pub xinput_handle: XInputHandle,
}

/// 设备配置文件的TOML结构
#[derive(Debug, Serialize, Deserialize)]
struct SupportedDevicesConfig {
    /// 支持的设备列表
    devices: Vec<DeviceInfo>,
}

// ---------------------- 枚举定义 ----------------------
/// 控制器类型分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerType {
    /// Xbox 系列控制器
    Xbox,
    /// PlayStation 系列控制器
    PlayStation,
    /// Nintendo Switch 控制器
    Switch,
    /// 北通(BETOP)系列控制器
    Betop,
    /// 其他未分类控制器
    Other,
}

// ---------------------- 全局静态变量 ----------------------
/// 全局应用句柄存储
#[allow(dead_code)]
static HANDLES: Lazy<Mutex<Option<Handles>>> = Lazy::new(|| Mutex::new(None));

/// 当前选中的控制器设备（多线程读多，写少）
#[allow(dead_code)]
pub static CURRENT_DEVICE: Lazy<RwLock<DeviceInfo>> = Lazy::new(|| {
    RwLock::new(DeviceInfo {
        name: "".into(),
        vendor_id: "".into(),
        product_id: None,
        device_path: None,
        controller_type: ControllerType::Other,
    })
});

/// 当前控制器采样数据（高频读取，偶尔写入）
pub static CONTROLLER_DATA: Lazy<RwLock<ControllerDatas>> =
    Lazy::new(|| RwLock::new(ControllerDatas::new()));

/// 自适应采样器实例（结构复杂，保持 Mutex）
#[allow(dead_code)]
pub static ADAPTER: Lazy<Mutex<AdaptiveSampler>> =
    Lazy::new(|| Mutex::new(AdaptiveSampler::new(200_000.0, 10.0)));

/// 全局 Gilrs 实例（外部库状态可能频繁修改，保守用 Mutex）
#[allow(dead_code)]
pub static GLOBAL_GILRS: Lazy<Mutex<Option<Gilrs>>> = Lazy::new(|| Mutex::new(None));

/// 支持的设备配置文件名称（常量，不变）
#[allow(dead_code)]
pub static SUPPORTED_DEVICES_FILE: &str = "supported_devices.toml";

/// 全局轮询频率 (Hz)（只读居多）
#[allow(dead_code)]
pub static FREQ: Lazy<RwLock<u32>> = Lazy::new(|| RwLock::new(125));

/// 采样率缓存值（只读居多）
#[allow(dead_code)]
pub static SAMPLING_RATE: Lazy<RwLock<f64>> = Lazy::new(|| RwLock::new(1000.0));

/// 轮询时间间隔 (秒)（频繁读，偶尔写）
#[allow(dead_code)]
pub static TIME_INTERVAL: Lazy<RwLock<f32>> = Lazy::new(|| RwLock::new(1.0));

// ---------------------- 控制器类型检测 ----------------------
/// 根据厂商ID识别控制器类型
///
/// # 参数
/// - `vid`: 厂商ID字符串 (16进制格式)
///
/// # 返回
/// 对应的 `ControllerType` 枚举值
pub fn detect_controller_type(vid: &str) -> ControllerType {
    match vid.to_ascii_lowercase().as_str() {
        "045e" => ControllerType::Xbox,        // Microsoft
        "054c" => ControllerType::PlayStation, // Sony
        "057e" => ControllerType::Switch,      // Nintendo
        "20bc" => ControllerType::Betop,       // BETOP
        _ => ControllerType::Other,
    }
}

// ---------------------- 配置管理 ----------------------
/// 生成默认支持的设备列表
///
/// 当配置文件不存在时使用此默认配置
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

/// 加载或创建设备配置文件
///
/// # 参数
/// - `path`: 配置文件相对路径
///
/// # 返回
/// 设备信息列表
///
/// # 行为
/// 1. 配置文件存在 -> 加载并解析
/// 2. 配置文件不存在 -> 创建默认配置
/// 3. 解析失败 -> 回退到默认配置
pub fn load_or_create_config(path: &str) -> Vec<DeviceInfo> {
    let config_path = xeno_utils::get_config_path(path);
    xeno_utils::ensure_config_dir();

    if config_path.exists() {
        match xeno_utils::read_toml_file::<SupportedDevicesConfig>(&config_path) {
            Ok(mut config) => {
                for device in &mut config.devices {
                    device.controller_type = detect_controller_type(&device.vendor_id);
                }
                config.devices
            }
            Err(e) => {
                log::error!("读取/解析配置文件失败: {e}");
                default_devices()
            }
        }
    } else {
        log::info!("🛠️ 配置文件不存在，正在生成默认配置: {:#?}", config_path);

        let default = default_devices();
        let config = SupportedDevicesConfig {
            devices: default.clone(),
        };

        if let Err(e) = xeno_utils::write_toml_file(&config_path, &config) {
            log::error!("写入默认配置文件失败: {e}");
        }

        default
    }
}

// ---------------------- 设备检测 ----------------------
/// 检测当前连接的设备并匹配支持列表
///
/// # 参数
/// - `config`: 支持的设备配置列表
///
/// # 返回
/// 已连接且被支持的设备列表（包含运行时信息）
pub fn list_supported_connected_devices(config: &[DeviceInfo]) -> Vec<DeviceInfo> {
    let api = match HidApi::new() {
        Ok(api) => api,
        Err(e) => {
            log::error!("初始化 hidapi 失败: {e}");
            return Vec::new();
        }
    };

    let mut supported_devices = Vec::new();

    // 遍历所有检测到的HID设备
    for device in api.device_list() {
        let vid = format!("{:04x}", device.vendor_id());
        let pid = format!("{:04x}", device.product_id());

        // 在配置中查找匹配项
        let matched = config.iter().find(|d| {
            d.vendor_id.eq_ignore_ascii_case(&vid)
                && match &d.product_id {
                    Some(pid_cfg) => pid_cfg.eq_ignore_ascii_case(&pid),
                    None => true,
                }
        });

        if let Some(_supported) = matched {
            // 构建完整的设备信息
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
/// 获取全局 Tauri 应用句柄
///
/// # Panics
/// 如果全局句柄未初始化会 panic
pub fn get_app_handle() -> AppHandle {
    HANDLES
        .lock()
        .unwrap()
        .as_ref()
        .expect("HANDLES not initialized")
        .app_handle
        .clone()
}

/// 获取 XInput 句柄 (Windows only)
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

/// 内部：获取支持的设备列表
fn _list_supported_devices() -> Vec<DeviceInfo> {
    let config = load_or_create_config(SUPPORTED_DEVICES_FILE);
    list_supported_connected_devices(&config)
}

/// 内部：查询可用设备
fn _query_devices() -> Vec<DeviceInfo> {
    _list_supported_devices()
}

/// 内部：按名称查找设备
fn _find_device_by_name(name: &str) -> Option<DeviceInfo> {
    _list_supported_devices()
        .into_iter()
        .find(|d| d.name == name)
}

// ---------------------- Tauri 命令接口 ----------------------

#[tauri::command]
pub fn get_controller_data() -> ControllerDatas {
    let controller_data = CONTROLLER_DATA.read().unwrap().clone();

    controller_data
}

/// 查询可用设备命令 (Tauri 前端调用)
///
/// 触发 "update_devices" 事件通知前端
#[tauri::command]
pub async fn query_devices(app: AppHandle) -> Vec<DeviceInfo> {
    let devices = _query_devices();
    if let Err(e) = app.emit("update_devices", devices.clone()) {
        log::error!("发送 update_devices 事件失败: {e}");
    }
    log::debug!("执行了 query_devices 命令");
    devices
}

/// 选择使用指定设备命令 (Tauri 前端调用)
#[tauri::command]
pub async fn use_device(device_name: String) -> bool {
    log::debug!("尝试使用设备: {device_name}");
    let device = _find_device_by_name(&device_name);
    match device {
        Some(device) => {
            let mut current_device = CURRENT_DEVICE.write().unwrap();
            *current_device = device;
            log::info!("✅ 使用设备: {}", current_device.name);
            true
        }
        None => {
            log::error!("❌ 未找到名为 '{device_name}' 的设备");
            false
        }
        _ => {
            log::error!("❌ 未知错误");
            false
        } // 无状态变化
    }
}

/// 断开当前设备命令 (Tauri 前端调用)
#[tauri::command]
pub async fn disconnect_device() -> bool {
    log::debug!("尝试断开设备连接");
    let mut current_device = CURRENT_DEVICE.write().unwrap();
    *current_device = default_devices()[0].clone();
    log::info!("✅ 已断开当前设备");
    true
}

/// 设置轮询频率命令 (Tauri 前端调用)
///
/// 同时更新相关参数：
/// - 全局频率值
/// - 采样率
/// - 时间间隔
#[tauri::command]
pub async fn set_frequency(freq: u32) {
    let freq = freq.clamp(1, 8000);
    let mut global_freq = FREQ.write().unwrap();
    let mut time_interval = TIME_INTERVAL.write().unwrap();
    let mut sample_rate = SAMPLING_RATE.write().unwrap();
    let adapter = ADAPTER.lock().unwrap();

    *global_freq = freq;
    *sample_rate = adapter.compute_sampling_rate(freq as f64);
    *time_interval = 1.0 / freq as f32;

    log::info!(
        "轮询频率: {} Hz ({}秒), 采样率: {:.2} Hz",
        *global_freq,
        *time_interval,
        *sample_rate
    );
}

// ---------------------- 设备轮询 ----------------------

fn _poll_other_controllers(gamepad: Gamepad) {
    // 检测按键状态


    let buttons = [
        (gamepad.is_pressed(Button::South), ControllerButtons::South, "South"),
        (gamepad.is_pressed(Button::East), ControllerButtons::East, "East"),
        (gamepad.is_pressed(Button::West), ControllerButtons::West, "West"),
        (gamepad.is_pressed(Button::North), ControllerButtons::North, "North"),

        (gamepad.is_pressed(Button::DPadDown), ControllerButtons::Down, "DPadDown"),
        (gamepad.is_pressed(Button::DPadLeft), ControllerButtons::Left, "DPadLeft"),
        (gamepad.is_pressed(Button::DPadRight), ControllerButtons::Right, "DPadRight"),
        (gamepad.is_pressed(Button::DPadUp), ControllerButtons::Up, "DPadUp"),

        (gamepad.is_pressed(Button::LeftTrigger), ControllerButtons::LB, "LB"),
        (gamepad.is_pressed(Button::RightTrigger), ControllerButtons::RB, "RB"),

        (gamepad.is_pressed(Button::LeftThumb), ControllerButtons::LStick, "LStick"),
        (gamepad.is_pressed(Button::RightThumb), ControllerButtons::RStick, "RStick"),

        (gamepad.is_pressed(Button::Select), ControllerButtons::Back, "Select"),
        (gamepad.is_pressed(Button::Start), ControllerButtons::Start, "Start"),
    ];

    for (pressed, button, name) in buttons {
        if pressed {
            log::debug!("{} 键被按下", name);
        }
        let mut controller_data = CONTROLLER_DATA.write().unwrap();
        controller_data.set_button(button, pressed);
    }

    log::debug!("---------------- {:#?}", gamepad.id());
    let left_stick_x = gamepad.axis_data(Axis::LeftStickX).unwrap().value();
    let left_stick_y = gamepad.axis_data(Axis::LeftStickY).unwrap().value();
    log::debug!("Left Stick X: {:#?}, Left Stick Y: {:#?}", left_stick_x, left_stick_y);

    let right_stick_x = gamepad.axis_data(Axis::RightStickX).unwrap().value();
    let right_stick_y = gamepad.axis_data(Axis::RightStickY).unwrap().value();
    log::debug!("Right Stick X: {:#?}, Right Stick Y: {:#?}", right_stick_x, right_stick_y);
    log::debug!("----------------");


}

/// 轮询非Xbox控制器状态
fn poll_other_controllers(device: &DeviceInfo) {
    let gilrs_guard = GLOBAL_GILRS.lock().unwrap();
    let gilrs = gilrs_guard.as_ref().unwrap();

    // 遍历所有已连接的游戏手柄
    for (_id, gamepad) in gilrs.gamepads() {
        let vid = format!("{:04x}", gamepad.vendor_id().unwrap());
        let pid = format!("{:04x}", gamepad.product_id().unwrap());

        // 匹配当前设备
        if vid.eq_ignore_ascii_case(&device.vendor_id)
            && pid.eq_ignore_ascii_case(device.product_id.as_deref().unwrap() )
        {
            _poll_other_controllers(gamepad);
        }
    }
}

/// 根据控制器类型分发轮询任务
fn poll_controller(device: &DeviceInfo) {
    match device.controller_type {
        // ControllerType::Xbox => poll_other_controllers(device),
        ControllerType::Xbox => xbox::poll_xbox_controller(device),
        _ => poll_other_controllers(device),
    }
}

// ---------------------- 后台任务 ----------------------
/// 启动设备发现后台任务
///
/// 每500ms扫描一次设备并发送更新事件
pub fn polling_devices() {
    // tauri::async_runtime::spawn(async move {
    //     log::info!("🛠️ 控制器监听已启动...");
    //     let app_handle = get_app_handle();
    //     loop {
    //         let devices = _query_devices();
    //         if let Err(e) = app_handle.emit("update_devices", devices.clone()) {
    //             log::error!("发送 update_devices 事件失败: {e}");
    //         }
    //         tokio::time::sleep(Duration::from_millis(500)).await;
    //     }
    // });
}

/// 主设备状态监听循环
pub fn listen() {
    thread::spawn(|| {
        log::info!("🎧 启动设备监听任务");
        let mut last_device: Option<DeviceInfo> = None;

        loop {
            let time_interval = *TIME_INTERVAL.read().unwrap();
            let current_device = CURRENT_DEVICE.read().unwrap().clone();

            // 设备连接状态跟踪
            let last_has_device = last_device.is_some();
            let current_has_device = current_device.device_path.is_some();

            match (last_has_device, current_has_device) {
                (false, true) => {
                    log::info!("🔌 连接新设备: {}", current_device.name);
                    last_device = Some(current_device.clone());
                }
                (true, true)
                    if last_device.as_ref().unwrap().device_path != current_device.device_path =>
                {
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

            // 执行设备状态轮询
            if let Some(device) = &last_device {
                poll_controller(device);
                mapping::map(device.clone(), *CONTROLLER_DATA.read().unwrap());
            }

            thread::sleep(Duration::from_secs_f32(time_interval));
        }
    });
}

pub fn _disconnect_device() {
    disconnect_device();
    let app_handle = get_app_handle();
    if let Err(e) = app_handle.emit("physical_connect_status", false) {
        log::error!("发送 physical_connect_status 事件失败: {e}");
    }
}

/// 初始化 Gilrs 事件监听线程
pub fn gilrs_listen() {
    thread::spawn(move || {
        let gilrs = Gilrs::new().expect("Failed to init Gilrs");
        {
            *GLOBAL_GILRS.lock().unwrap() = Some(gilrs);
        }

        loop {
            // let mut disconnect_event = false;

            if let Some(gilrs) = GLOBAL_GILRS.lock().unwrap().as_mut() {
                // 清空事件队列但不处理
                while let Some(Event { event, id, .. }) = gilrs.next_event() {
                    if event == EventType::Disconnected {
                        #[cfg(target_os = "windows")]
                        {
                            let device = CURRENT_DEVICE.read().unwrap();
                            if device.controller_type != ControllerType::Xbox {
                                _disconnect_device();
                            }
                        }

                        #[cfg(not(target_os = "windows"))]
                        _disconnect_device();
                    }
                }
            }

            thread::sleep(Duration::from_secs_f32(
                1.0 / *SAMPLING_RATE.read().unwrap() as f32,
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
///
/// 启动三个核心任务：
/// 1. Gilrs 事件监听
/// 2. 设备发现轮询
/// 3. 主设备状态监听
pub fn initialize(app_handle: AppHandle) {
    log::debug!("初始化控制器模块");
    
    query_needed_handle(app_handle);

    gilrs_listen();
    listen();

    polling_devices();
}
