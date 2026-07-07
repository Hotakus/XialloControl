#![allow(dead_code)]

pub mod calibrate;
pub mod datas;
pub mod logic;
pub mod xbox;
pub mod ps4;

// ---------------------- 外部依赖 ----------------------
use crate::adaptive_sampler::AdaptiveSampler;
use crate::controller::datas::{CompactPressureDatas, ControllerButtons, ControllerDatas};
use crate::{controller, mapping, preset, xeno_utils};
use gilrs::{Axis, Event, EventType, Gamepad, Gilrs, GilrsBuilder};
use hidapi::HidApi;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, RwLock};
use std::time::Instant;
use std::{thread, time::Duration};
use tauri::{AppHandle, Emitter};

use crate::setting::{self, get_setting, LastConnectedDevice};
#[cfg(target_os = "windows")]
use rusty_xinput::XInputHandle;
use uuid::Uuid;

use std::collections::HashMap;

// --- 副预设切换状态 ---
/// 用于追踪 Toggle 模式下的副预设激活状态
static IS_SUB_PRESET_ACTIVE: Lazy<RwLock<bool>> = Lazy::new(|| RwLock::new(false));
/// 用于检测按键单击事件 (按下后立即释放)
static TOGGLE_BUTTON_LAST_STATE: Lazy<RwLock<bool>> = Lazy::new(|| RwLock::new(false));

// ---------------------- 结构体定义 ----------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JoystickSource {
    LeftStick,
    RightStick,
}

use crate::controller::datas::JoystickRotation;

/// 摇杆旋转的物理状态 (用于在 controller 模块内部追踪)
#[derive(Clone, Debug)]
pub struct JoystickRotationState {
    pub last_angle: f32,
    pub was_active: bool,
    pub current_rotation: JoystickRotation,
    pub last_rotation_time: Instant,
    pub accumulated_angle_delta: f32,
}

impl Default for JoystickRotationState {
    fn default() -> Self {
        Self {
            last_angle: 0.0,
            was_active: false,
            current_rotation: JoystickRotation::None,
            last_rotation_time: Instant::now(),
            accumulated_angle_delta: 0.0,
        }
    }
}

/// 游戏控制器设备信息
#[derive(Debug, Eq, Serialize, Deserialize, Clone, PartialEq)]
pub struct DeviceInfo {
    /// 设备显示名称
    pub name: String,
    /// 厂商ID (16进制字符串)
    pub vendor_id: String,
    /// 产品ID (16进制字符串，可选)
    pub product_id: Option<String>,
    /// 子产品ID (16进制字符串，可选)
    pub sub_product_id: Option<String>,
    /// UUID是否无效 (部分设备可能无UUID)
    pub uuid_is_invalid: bool,
    /// 设备路径 (运行时检测)
    pub device_path: Option<String>,
    /// 控制器类型分类
    pub controller_type: ControllerType,
}

impl DeviceInfo {
    pub fn new(name: String, vendor_id: String, controller_type: ControllerType) -> Self {
        DeviceInfo {
            name,
            vendor_id,
            product_id: None,
            sub_product_id: None,
            uuid_is_invalid: false,
            device_path: None,
            controller_type,
        }
    }
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
#[derive(Hash)]
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
        sub_product_id: None,
        uuid_is_invalid: true,
        device_path: None,
        controller_type: ControllerType::Other,
    })
});

/// 当前控制器采样数据（高频读取，偶尔写入）
pub static CONTROLLER_DATA: Lazy<RwLock<ControllerDatas>> =
    Lazy::new(|| RwLock::new(ControllerDatas::new()));

/// 原始控制器采样数据，专用于校准
pub static RAW_CONTROLLER_DATA: Lazy<RwLock<ControllerDatas>> =
    Lazy::new(|| RwLock::new(ControllerDatas::new()));

pub static PREV_CONTROLLER_DATA: Lazy<RwLock<ControllerDatas>> =
    Lazy::new(|| RwLock::new(ControllerDatas::new()));
pub static PREV_BTN_DATA: Lazy<RwLock<u32>> =
    Lazy::new(|| RwLock::new(0));

pub static PREV_PRESSURE_DATA: Lazy<RwLock<CompactPressureDatas>> =
    Lazy::new(|| RwLock::new(CompactPressureDatas::new()));

/// 全局摇杆旋转物理状态 (由 controller 模块独占)
pub static JOYSTICK_ROTATION_STATES: Lazy<RwLock<HashMap<JoystickSource, JoystickRotationState>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

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
        "20bc" | "11c0" | "8380" => ControllerType::Betop, // BETOP（多 VID）
        // 以下厂商手柄多数遵循 Xbox 布局（XInput 兼容），归入 Xbox 类型复用其 layout
        // VID 来源：Linux kernel hid-ids.h + mdqinc/SDL_GameControllerDB
        "1532"   // Razer
        | "2dc8"  // 8BitDo
        | "046d"  // Logitech
        | "0f0d"  // HORI
        | "044f"  // Thrustmaster
        | "0738"  // Mad Catz
        | "28de"  // Valve Steam Controller
        | "04b4"  // Flydigi（Cypress Semiconductor 子授权芯片）
        | "2f24"  // GameSir（ShenZhen HuiJiaZhi 代工）
        | "3537"  // GameSir（自主 VID — G7 Pro / T4 Kaleid / Cyclone / Kaleid Flux 等，均 XInput 兼容）
        | "146b"  // Nacon（Bigben Interactive / NACON SA）
        | "0079"  // Mayflash（Shenzhen Longshengwei，同 DragonRise）
        => ControllerType::Xbox,
        _ => ControllerType::Other,
    }
}

// ---------------------- 配置管理 ----------------------
/// 生成默认支持的设备列表
///
/// 当配置文件不存在时使用此默认配置
fn default_devices() -> Vec<DeviceInfo> {
    vec![
        DeviceInfo::new("UNKNOWN CONTROLLER".into(), "0000".into(), ControllerType::Other),
        DeviceInfo::new("Any Xbox Controller".into(), "045e".into(), ControllerType::Xbox),
        DeviceInfo::new("DualShock 4 (PS4)".into(), "054c".into(), ControllerType::PlayStation),
        DeviceInfo::new("DualSense (PS5)".into(), "054c".into(), ControllerType::PlayStation),
        DeviceInfo::new("Switch Pro".into(), "057e".into(), ControllerType::Switch),
        DeviceInfo::new("[ BETOP CONTROLLER ]".into(), "20bc".into(), ControllerType::Other),
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
        log::info!("🛠️ 配置文件不存在，正在生成默认配置: {config_path:#?}");

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
fn list_controllers_from_gilrs() -> Vec<DeviceInfo> {
    let gilrs_guard = GLOBAL_GILRS.lock().unwrap_or_else(|e| e.into_inner());
    let gilrs = match gilrs_guard.as_ref() {
        Some(g) => g,
        None => return Vec::new(),
    };

    let mut devices = Vec::new();
    for (_id, gamepad) in gilrs.gamepads() {
        let vid_opt = gamepad.vendor_id();
        let pid_opt = gamepad.product_id();

        if let (Some(vid), Some(pid)) = (vid_opt, pid_opt) {
            let vid_str = format!("{vid:04x}");
            let pid_str = format!("{pid:04x}");
            let uuid = Uuid::from_bytes(gamepad.uuid());

            let device_info = DeviceInfo {
                name: gamepad.name().to_string(),
                vendor_id: vid_str.clone(),
                product_id: Some(pid_str),
                sub_product_id: None,
                uuid_is_invalid: uuid.is_nil(),
                device_path: None,
                controller_type: detect_controller_type(&vid_str),
            };
            log::info!(
                "检测到设备: {} (vid:{}, pid:{}, uuid_invalid:{}) 映射源: {:?}",
                device_info.name,
                device_info.vendor_id,
                device_info.product_id.as_deref().unwrap_or("unknown"),
                device_info.uuid_is_invalid,
                gamepad.mapping_source()
            );
            devices.push(device_info);
        }
    }

    devices
}

fn list_controllers_should_manage() -> Vec<DeviceInfo> {
    let mut devices = list_controllers_from_gilrs();
    let hidapi = match HidApi::new() {
        Ok(api) => api,
        Err(e) => {
            log::error!("初始化 hidapi 失败: {e}");
            return Vec::new();
        }
    };

    // 遍历所有检测到的HID设备，并将路径信息和子PID补全
    for device in hidapi.device_list() {
        let vid = format!("{:04x}", device.vendor_id());
        let pid = format!("{:04x}", device.product_id());

        for d in devices.iter_mut() {
            if d.vendor_id.eq_ignore_ascii_case(&vid) {
                log::error!("({}/{}) - ({vid},{pid})", d.vendor_id, d.product_id.as_deref().unwrap_or("Unknown"));
                d.sub_product_id = Some(pid.clone());
                d.device_path = Some(device.path().to_string_lossy().to_string());
                if d.device_path.is_none() {
                    log::warn!("手柄路径缺失：{d:#?}");
                }
            }
        }
    }

    devices.into_iter().filter(|d| {
        !((d.vendor_id.eq("0000") || d.vendor_id.is_empty()) && d.product_id.is_none())
    }).collect()
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

/// 内部：查询可用设备
fn _query_devices() -> Vec<DeviceInfo> {
    list_controllers_should_manage()
}

/// 内部：按名称查找设备
fn _find_device_by_name(name: &str) -> Option<DeviceInfo> {
    let devices = _query_devices();
    devices.into_iter().find(|d| d.name == name)
}

// ---------------------- Tauri 命令接口 ----------------------

#[tauri::command]
pub fn get_controller_data() -> ControllerDatas {
    *CONTROLLER_DATA.read().unwrap()
}

/// 查询可用设备命令 (Tauri 前端调用)
///
/// 触发 "update_devices" 事件通知前端
#[tauri::command]
pub fn query_devices(app: AppHandle) -> Vec<DeviceInfo> {
    let devices = _query_devices();
    if let Err(e) = app.emit("update_devices", devices.clone()) {
        log::error!("发送 update_devices 事件失败: {e}");
    }
    log::debug!("执行了 query_devices 命令");
    devices
}

/// 更新设置中上次连接的设备信息
fn update_last_connected_device_setting(device_info: Option<DeviceInfo>) {
    let mut settings = get_setting();
    settings.last_connected_device = device_info.map(|d| LastConnectedDevice {
        vid: u16::from_str_radix(&d.vendor_id, 16).unwrap_or(0),
        pid: u16::from_str_radix(&d.product_id.unwrap_or_default(), 16).unwrap_or(0),
        sub_pid: u16::from_str_radix(&d.sub_product_id.unwrap_or_default(), 16).unwrap_or(0),
    });
    let app_handle = get_app_handle();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = setting::update_settings(app_handle, settings).await {
            log::error!("保存上次连接设备信息失败: {e:?}");
        }
    });
}

/// 选择使用指定设备命令 (Tauri 前端调用)
#[tauri::command]
pub fn use_device(device_name: String) -> bool {
    log::debug!("尝试使用设备: {device_name}");
    let device = _find_device_by_name(&device_name);
    match device {
        Some(mut device_info) => {
                // 如果 hidapi 未检测到此设备（device_path 为 None），使用 WGI 标识符作为 fallback。
                // 某些 XInput 兼容手柄（如 GameSir G7 Pro）对 hidapi 不可见，但 WGI/gilrs 能检测到。
                // 没有 device_path 会导致 listen() 轮询线程无法启动（issue #15）。
                if device_info.device_path.is_none() {
                    device_info.device_path = Some(format!("wgi:{}", device_info.vendor_id));
                    log::warn!(
                        "hidapi 未检测到此设备 (vid:{})，使用 WGI 标识符作为设备路径",
                        device_info.vendor_id
                    );
                }
                let mut current_device = CURRENT_DEVICE.write().unwrap();
            *current_device = device_info.clone();
            log::info!("✅ 使用设备: {}", current_device.name);

            // 加载与此设备关联的校准数据
            crate::controller::calibrate::load_calibration(&device_info);

            drop(current_device); // 显式释放锁
            update_last_connected_device_setting(Some(device_info));
            true
        }
        None => {
            log::error!("❌ 未找到名为 '{device_name}' 的设备");
            false
        }
    }
}

#[tauri::command]
pub fn disconnect_device() -> bool {
    log::debug!("尝试断开设备连接");
    let mut current_device = CURRENT_DEVICE.write().unwrap();
    *current_device = default_devices()[0].clone();
    log::info!("✅ 已断开当前设备");

    // 重置全局校准数据
    crate::controller::calibrate::reset_calibration();

    drop(current_device); // 显式释放锁
    update_last_connected_device_setting(None);
    true
}

/// 断开当前设备命令 (Tauri 前端调用)
#[tauri::command]
pub fn physical_disconnect_device() -> bool {
    log::debug!("------ physical_disconnect_device");
    let app_handle = get_app_handle();
    let dname = {
        let device = CURRENT_DEVICE.read().unwrap();
        device.name.clone()
    };
    log::info!("尝试断开设备: {dname}");
    if let Err(e) = app_handle.emit("physical_connect_status", dname) {
        log::error!("发送 physical_connect_status 事件失败: {e}");
        return false;
    }
    disconnect_device()
}


/// 设置轮询频率命令 (Tauri 前端调用)
///
/// 同时更新相关参数：
/// - 全局频率值
/// - 采样率
/// - 时间间隔
#[tauri::command]
pub fn set_frequency(freq: u32) {
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

#[tauri::command]
pub fn need_rumble() {

}

// ---------------------- 设备轮询 ----------------------

pub fn pack_and_send_data(controller_data: &ControllerDatas) {
    let mut prev_controller_data = PREV_CONTROLLER_DATA.write().unwrap();
    if controller_data.eq(&prev_controller_data) {
        // 无变化，不发送数据
        return;
    }

    // 数据有变化则进一步比较具体值
    // 按键数据变化
    let app_handle = get_app_handle();
    let compact_data = controller_data.as_compact();
    if let Err(e) = app_handle.emit("update_controller_compact_datas", compact_data) {
        log::error!("按键数据发送失败: {e}");
    }
    drop(app_handle);

    *prev_controller_data = *controller_data;
}


fn _poll_other_controllers(gamepad: Gamepad) {
    // 检测按键状态
    let mut controller_data = CONTROLLER_DATA.write().unwrap();

    let buttons = [
        (gamepad.is_pressed(gilrs::Button::South), ControllerButtons::South),
        (gamepad.is_pressed(gilrs::Button::East), ControllerButtons::East),
        (gamepad.is_pressed(gilrs::Button::West), ControllerButtons::West),
        (gamepad.is_pressed(gilrs::Button::North), ControllerButtons::North),
        (gamepad.is_pressed(gilrs::Button::DPadDown), ControllerButtons::Down),
        (gamepad.is_pressed(gilrs::Button::DPadLeft), ControllerButtons::Left),
        (gamepad.is_pressed(gilrs::Button::DPadRight), ControllerButtons::Right),
        (gamepad.is_pressed(gilrs::Button::DPadUp), ControllerButtons::Up),
        (gamepad.is_pressed(gilrs::Button::LeftTrigger), ControllerButtons::LB),
        (gamepad.is_pressed(gilrs::Button::RightTrigger), ControllerButtons::RB),
        (gamepad.is_pressed(gilrs::Button::LeftThumb), ControllerButtons::LStick),
        (gamepad.is_pressed(gilrs::Button::RightThumb), ControllerButtons::RStick),
        (gamepad.is_pressed(gilrs::Button::Select), ControllerButtons::Back),
        (gamepad.is_pressed(gilrs::Button::Start), ControllerButtons::Start),
        (gamepad.is_pressed(gilrs::Button::Mode), ControllerButtons::Guide),
    ];

    let pressed_count: usize = buttons.iter().filter(|(p, _)| *p).count();

    // 125Hz 高频路径，节流输出：首条打印，后续每 125 次（≈1s）汇总一次
    static POLL_CNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let cnt = POLL_CNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if cnt == 0 || cnt % 125 == 0 {
        log::debug!(
            "gilrs 轮询 #{cnt}: 按键按下 {pressed_count}/15, 左摇杆=({:.3},{:.3}) 右摇杆=({:.3},{:.3})",
            gamepad.axis_data(Axis::LeftStickX).map_or(0.0, |d| d.value()),
            gamepad.axis_data(Axis::LeftStickY).map_or(0.0, |d| d.value()),
            gamepad.axis_data(Axis::RightStickX).map_or(0.0, |d| d.value()),
            gamepad.axis_data(Axis::RightStickY).map_or(0.0, |d| d.value()),
        );
    }
    // 诊断：WGI nil UUID 回退后，检查 gilrs 层逻辑按钮状态
    if cnt == 1 || cnt % 250 == 0 {
        log::debug!(
            "diag: South={} East={} West={} North={} LB={} RB={} LT2={:.3} RT2={:.3} DPadU={} DPadD={} LStk={} RStk={} Sel={} Start={}",
            gamepad.is_pressed(gilrs::Button::South),
            gamepad.is_pressed(gilrs::Button::East),
            gamepad.is_pressed(gilrs::Button::West),
            gamepad.is_pressed(gilrs::Button::North),
            gamepad.is_pressed(gilrs::Button::LeftTrigger),
            gamepad.is_pressed(gilrs::Button::RightTrigger),
            gamepad.button_data(gilrs::Button::LeftTrigger2).map_or(0.0, |d| d.value()),
            gamepad.button_data(gilrs::Button::RightTrigger2).map_or(0.0, |d| d.value()),
            gamepad.is_pressed(gilrs::Button::DPadUp),
            gamepad.is_pressed(gilrs::Button::DPadDown),
            gamepad.is_pressed(gilrs::Button::LeftThumb),
            gamepad.is_pressed(gilrs::Button::RightThumb),
            gamepad.is_pressed(gilrs::Button::Select),
            gamepad.is_pressed(gilrs::Button::Start),
        );
    }

    for (pressed, button) in buttons {
        controller_data.set_button(button, pressed);
    }

    let raw_lx = gamepad.axis_data(Axis::LeftStickX).map_or(0.0, |d| d.value());
    let raw_ly = gamepad.axis_data(Axis::LeftStickY).map_or(0.0, |d| d.value());
    let raw_rx = gamepad.axis_data(Axis::RightStickX).map_or(0.0, |d| d.value());
    let raw_ry = gamepad.axis_data(Axis::RightStickY).map_or(0.0, |d| d.value());

    // // 将原始数据写入 RAW_CONTROLLER_DATA 供校准线程使用
    // {
    //     let mut raw_data = RAW_CONTROLLER_DATA.write().unwrap();
    //     raw_data.left_stick.x = raw_lx;
    //     raw_data.left_stick.y = raw_ly;
    //     raw_data.right_stick.x = raw_rx;
    //     raw_data.right_stick.y = raw_ry;
    // }

    // let (final_lx, final_ly, final_rx, final_ry) = get_calibrated_stick_values(raw_lx, raw_ly, raw_rx, raw_ry);

    controller_data.left_stick.x = raw_lx;
    controller_data.left_stick.y = raw_ly;
    controller_data.right_stick.x = raw_rx;
    controller_data.right_stick.y = raw_ry;

    controller_data.right_stick.is_pressed = gamepad.is_pressed(gilrs::Button::RightThumb);
    controller_data.left_stick.is_pressed = gamepad.is_pressed(gilrs::Button::LeftThumb);

    controller_data.left_trigger.value = gamepad.button_data(gilrs::Button::LeftTrigger2)
                                                .map(|data| data.value())
                                                .unwrap_or(0.0);
    controller_data.right_trigger.value = gamepad.button_data(gilrs::Button::RightTrigger2)
                                                 .map(|data| data.value())
                                                 .unwrap_or(0.0);
}

/// 轮询非Xbox控制器状态
fn poll_other_controllers(device: &DeviceInfo) {
    let gilrs_guard = GLOBAL_GILRS.lock().unwrap();
    let gilrs = gilrs_guard.as_ref().unwrap();

    let mut found = false;

    static POLL_OTHER_CNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let cnt = POLL_OTHER_CNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // 遍历所有已连接的游戏手柄
    for (_id, gamepad) in gilrs.gamepads() {
        let vid = format!("{:04x}", gamepad.vendor_id().unwrap());
        let pid = format!("{:04x}", gamepad.product_id().unwrap());

        // 首条打印 gamepad 列表，后续每 125 次（≈1s）汇总
        if cnt == 0 || cnt % 125 == 0 {
            log::debug!(
                "gilrs gamepad[{}]: vid={vid}, pid={pid}, target_vid={}, target_pid={}",
                _id,
                device.vendor_id,
                device.product_id.as_deref().unwrap_or("?")
            );
        }

        // 匹配当前设备
        if vid.eq_ignore_ascii_case(&device.vendor_id)
            && device.product_id.as_deref().is_some_and(|d_pid| pid.eq_ignore_ascii_case(d_pid))
        {
            found = true;
            _poll_other_controllers(gamepad);
        }
    }

    if !found && (cnt == 0 || cnt % 125 == 0) {
        log::warn!(
            "gilrs gamepads 中未找到匹配设备 (target: vid={}, pid={})",
            device.vendor_id,
            device.product_id.as_deref().unwrap_or("?")
        );
    }
}

/// 根据控制器类型分发轮询任务
fn poll_controller(device: &DeviceInfo) {
    match device.controller_type {
        // Xbox 控制器在 Windows 下优先走 XInput API（性能更好，不依赖 gilrs 事件队列）
        // 若 XInput 未找到匹配设备（如第三方手柄仅 WGI 可见），回退 gilrs
        ControllerType::Xbox => {
            #[cfg(target_os = "windows")]
            {
                if !xbox::poll_xbox_controller(device) {
                    static FALLBACK_CNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
                    let fc = FALLBACK_CNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if fc % 125 == 0 {
                        log::debug!("XInput 未匹配，回退 gilrs 轮询");
                    }
                    poll_other_controllers(device);
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                poll_other_controllers(device)
            }
        }
        ControllerType::PlayStation => {
            // ps4::poll_ps4_controller(device);
            poll_other_controllers(device);
        }
        _ => {
            // 非标准控制器统一走 gilrs（依赖 SDL DB 映射，gilrs-core UUID 修复后生效）
            poll_other_controllers(device)
        }
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

/// 处理预设切换决策, 返回是否应该使用副预设
fn handle_preset_switching_decision() -> bool {
    let controller_datas = CONTROLLER_DATA.read().unwrap();
    let main_preset = preset::get_current_preset();
    let sub_preset_guard = preset::CURRENT_SUB_PRESET.read().unwrap();

    let mut use_sub_preset = false;

    // 必须存在副预设，并且主预设有切换配置
    if sub_preset_guard.is_some() {
        if let (Some(button_name), Some(mode)) = (
            &main_preset.items.sub_preset_switch_button,
            &main_preset.items.sub_preset_switch_mode,
        ) {
            let layout_map = mapping::get_current_controller_layout_map();
            if let Some(button_enum) = layout_map.get(button_name.as_str()) {
                let is_button_pressed = controller_datas.get_button(*button_enum);

                match mode.as_str() {
                    "Hold" => {
                        use_sub_preset = is_button_pressed;
                    }
                    "Toggle" => {
                        {
                            let mut last_state = TOGGLE_BUTTON_LAST_STATE.write().unwrap();
                            if is_button_pressed && !*last_state {
                                let mut active = IS_SUB_PRESET_ACTIVE.write().unwrap();
                                *active = !*active;
                            }
                            *last_state = is_button_pressed;
                        }
                        use_sub_preset = *IS_SUB_PRESET_ACTIVE.read().unwrap();
                    }
                    _ => {}
                }
            }
        }
    }

    // 如果没有副预设, 确保 toggle 状态被重置
    if sub_preset_guard.is_none() {
        let mut active = IS_SUB_PRESET_ACTIVE.write().unwrap();
        if *active {
            *active = false;
        }
    }
    
    use_sub_preset
}


/// 主设备状态监听循环
pub fn listen() {
    thread::spawn(|| {
        log::info!("🎧 启动设备监听任务");
        let mut last_device: Option<DeviceInfo> = None;

        loop {
            let time_start = Instant::now();

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
                        log::info!("❌ 设备断开: {} ({}/{}/{})",
                        device.name, device.vendor_id,
                        device.product_id.as_deref().unwrap_or("Unknown"),
                        device.sub_product_id.as_deref().unwrap_or("Unknown"));
                    }
                    last_device = None;
                }
                _ => (), // 无状态变化
            }

            // 执行设备状态轮询
            if let Some(device) = &last_device {
                poll_controller(device);

                // 一次性获取锁，减少锁操作
                let mut controller_data = CONTROLLER_DATA.write().unwrap();
                logic::apply_deadzone(&mut controller_data);
                logic::check_sticks_rotation(&mut controller_data);

                let data_snapshot = *controller_data;
                drop(controller_data); // 释放写锁

                pack_and_send_data(&data_snapshot);
                mapping::handle_mouse_movement(&data_snapshot);

                let use_sub_preset = handle_preset_switching_decision();
                mapping::map(&mut CONTROLLER_DATA.write().unwrap(), use_sub_preset);
            }

            let elapsed = time_start.elapsed();
            // log::info!("elapsed time: {:#?}", elapsed);
            let time_interval = *TIME_INTERVAL.read().unwrap();
            thread::sleep(Duration::from_secs_f32(time_interval));
        }
    });
}

/// 初始化 Gilrs 事件监听线程
pub fn gilrs_listen() {
    thread::spawn(move || {
        let gilrs = GilrsBuilder::new()
        .add_mappings(include_str!("gamecontrollerdb_ext.txt"))
        .build()
        .expect("Failed to init Gilrs");
        {
            *GLOBAL_GILRS.lock().unwrap() = Some(gilrs);
        }

        loop {
            if let Some(gilrs) = GLOBAL_GILRS.lock().unwrap_or_else(|poisoned| {
                log::warn!("GLOBAL_GILRS 互斥锁已被污染，正在恢复...");
                poisoned.into_inner()
            }).as_mut() {
                // 清空事件队列但不处理
                static EVT_CNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
                while let Some(Event { event, id, .. }) = gilrs.next_event_blocking(Some(Duration::from_millis(1))) {
                    let n = EVT_CNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if n == 0 || n % 125 == 0 {
                        log::debug!("gilrs_listen 事件 #{n}: id={id} event={event:?}");
                    }
                    let _ = id;
                    if event == EventType::Disconnected {
                        let device = CURRENT_DEVICE.read().unwrap().clone();
                        let controller_type = device.controller_type;
                        if device.eq(&default_devices()[0].clone()) {
                            log::warn!("设备已断开，跳过处理");
                            continue;
                        }
                        drop(device);

                        #[cfg(target_os = "windows")]
                        if controller_type != ControllerType::Xbox {
                            log::warn!("---- 检测到设备断开，尝试物理断开设备");
                            physical_disconnect_device();
                        }

                        #[cfg(not(target_os = "windows"))]
                        physical_disconnect_device();
                    }
                    // if let EventType::AxisChanged(axis, value, code) = event {
                    //     log::info!("Axis {:?} changed: {}", axis, value);
                    // }
                    // if let EventType::ButtonChanged(b, v, code) = event {
                    //     log::info!("Button {:#?}, value {:#?} ({:#?})", b, v, code);
                    // }
                }
            }

            // 将采样率设置为基本轮询频率的至少两倍,以保证保证 gilrs 设备数据的准确准时读取
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

    let setting = get_setting();
    set_frequency(setting.polling_frequency);

    query_needed_handle(app_handle);
    gilrs_listen();
    listen();
    polling_devices();
}

/// 尝试自动连接上次连接的设备
#[tauri::command]
pub fn try_auto_connect_last_device(app_handle: AppHandle) {
    let settings = get_setting();
    if settings.remember_last_connection {
        if let Some(last_device) = settings.last_connected_device {
            log::info!("尝试连接上次连接的设备: {last_device:?}");
            let devices = query_devices(app_handle.clone()); // query_devices 现在是同步的
            if let Some(device_info) = devices.into_iter().find(|d| {
                let last_vid_str = format!("{:04x}", last_device.vid);
                let last_pid_str = format!("{:04x}", last_device.pid);
                let last_sub_pid_str = format!("{:04x}", last_device.sub_pid);

                let vid_matches = d.vendor_id == last_vid_str;

                let pid_matches = if last_device.pid == 0 {
                    true
                } else {
                    d.product_id.as_deref().is_some_and(|pid| pid == last_pid_str)
                };

                let sub_pid_matches = if last_device.sub_pid == 0 {
                    true
                } else {
                    d.sub_product_id.as_deref().is_some_and(|sub_pid| sub_pid == last_sub_pid_str)
                };

                // log::debug!("匹配检查: DeviceInfo {:?} vs LastConnectedDevice {:?}", d, last_device);
                // log::debug!("  VID: {} == {} -> {}", d.vendor_id, last_vid_str, vid_matches);
                // log::debug!("  PID: {:?} == {} -> {}", d.product_id, last_pid_str, pid_matches);
                // log::debug!("  SubPID: {:?} == {} -> {}", d.sub_product_id, last_sub_pid_str, sub_pid_matches);
                // log::debug!("  总匹配: {}", vid_matches && (pid_matches || sub_pid_matches));

                vid_matches && (pid_matches || sub_pid_matches)
            }) {
                log::info!("找到匹配的设备，尝试连接: {device_info:?}");
                if use_device(device_info.name.clone()) { // use_device 现在是同步的
                    log::info!("成功自动连接上次设备");
                    if let Err(e) = app_handle.emit("auto_connect_success", device_info) {
                        log::error!("发送 auto_connect_success 事件失败: {e}");
                    }
                } else {
                    log::error!("自动连接上次设备失败");
                }
            } else {
                log::warn!("未找到上次连接的设备: {last_device:?}");
            }
        } else {
            log::info!("记住上次连接状态已启用，但没有上次连接的设备信息。");
        }
    }
}
