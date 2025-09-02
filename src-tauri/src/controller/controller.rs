#![allow(dead_code)]

// ---------------------- 外部依赖 ----------------------
use crate::adaptive_sampler::AdaptiveSampler;
use crate::controller::datas::{CompactPressureDatas, ControllerButtons, ControllerDatas};
use crate::{mapping, preset, xeno_utils};
use gilrs::{Axis, Event, EventType, Gamepad, Gilrs};
use hidapi::HidApi;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, RwLock};
use std::time::Instant;
use std::{thread, time::Duration};
use tauri::{AppHandle, Emitter};

use crate::controller::xbox;
use crate::setting::{self, get_setting, LastConnectedDevice, AppSettings};
#[cfg(target_os = "windows")]
use rusty_xinput::XInputHandle;
use uuid::Uuid;

use std::collections::HashMap;
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
    last_angle: f32,
    was_active: bool,
    current_rotation: JoystickRotation,
    last_rotation_time: Instant,
}

impl Default for JoystickRotationState {
    fn default() -> Self {
        Self {
            last_angle: 0.0,
            was_active: false,
            current_rotation: JoystickRotation::None,
            last_rotation_time: Instant::now(),
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

    let mut connected_devices = Vec::new();
    let gilrs_guard = GLOBAL_GILRS.lock().unwrap();
    let gilrs = gilrs_guard.as_ref().unwrap();

    for (id, gamepad) in gilrs.gamepads() {
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
                device_path: Some(format!("{id:?}")),
                controller_type: detect_controller_type(&vid_str),
            };
            connected_devices.push(device_info);
        }
    }

    // 遍历所有检测到的HID设备
    for device in api.device_list() {
        let vid = format!("{:04x}", device.vendor_id());
        let pid = format!("{:04x}", device.product_id());

        for d in connected_devices.iter_mut() {
            if d.vendor_id.eq_ignore_ascii_case(&vid) {
                d.sub_product_id = Some(pid.clone());
            }
        }
    }

    connected_devices
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
            log::error!("保存上次连接设备信息失败: {:?}", e);
        }
    });
}

/// 选择使用指定设备命令 (Tauri 前端调用)
#[tauri::command]
pub fn use_device(device_name: String) -> bool {
    log::debug!("尝试使用设备: {device_name}");
    let device = _find_device_by_name(&device_name);
    match device {
        Some(device_info) => {
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

pub(crate) fn get_calibrated_stick_values(raw_lx: f32, raw_ly: f32, raw_rx: f32, raw_ry: f32) -> (f32, f32, f32, f32) {
    let preset = preset::get_current_preset();
    let cali_data = crate::controller::calibrate::get_current_calibration();

    let (lx, ly) = crate::controller::calibrate::apply_calibration(
        raw_lx,
        raw_ly,
        preset.items.deadzone_left,
        &cali_data.left_stick,
    );

    let (rx, ry) = crate::controller::calibrate::apply_calibration(
        raw_rx,
        raw_ry,
        preset.items.deadzone,
        &cali_data.right_stick,
    );
    (lx, ly, rx, ry)
}


/// 通用函数: 计算并更新单个摇杆的旋转状态, 并返回旋转状态
pub fn update_joystick_rotation_state(
    source: JoystickSource,
    x: f32,
    y: f32,
) -> JoystickRotation {
    const ROTATION_THRESHOLD: f32 = 0.01; // 旋转检测的最小角度变化阈值
    const JUMP_THRESHOLD: f32 = std::f32::consts::PI; // 角度跳变的阈值, 约180度
    const ROTATION_TIMEOUT_MS: u128 = 50; // 旋转状态维持的超时时间 (毫秒)

    let mut rotation_states = JOYSTICK_ROTATION_STATES.write().unwrap();
    let mut state = rotation_states.remove(&source).unwrap_or_default();

    let is_active = x != 0.0 || y != 0.0;

    if is_active {
        let angle = (-y).atan2(x);
        if state.was_active {
            let mut delta = angle - state.last_angle;
            if delta > std::f32::consts::PI { delta -= 2.0 * std::f32::consts::PI; }
            else if delta < -std::f32::consts::PI { delta += 2.0 * std::f32::consts::PI; }

            // 检查是否为大幅度跳变 (回弹), 如果是, 则重置状态
            if delta.abs() > JUMP_THRESHOLD {
                state.current_rotation = JoystickRotation::None;
            } else if delta > ROTATION_THRESHOLD {
                state.current_rotation = JoystickRotation::Clockwise;
                state.last_rotation_time = Instant::now();
            } else if delta < -ROTATION_THRESHOLD {
                state.current_rotation = JoystickRotation::CounterClockwise;
                state.last_rotation_time = Instant::now();
            }
        }
        state.last_angle = angle;
    } else {
        // 如果摇杆回到死区, 直接重置状态
        state.current_rotation = JoystickRotation::None;
    }
    
    // 超时检查: 如果距离上次有效旋转超过一定时间, 则认为旋转已停止
    if state.last_rotation_time.elapsed().as_millis() > ROTATION_TIMEOUT_MS {
        state.current_rotation = JoystickRotation::None;
    }

    state.was_active = is_active;
    let rotation = state.current_rotation;
    rotation_states.insert(source, state);
    rotation
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

    for (pressed, button) in buttons {
        controller_data.set_button(button, pressed);
    }

    let raw_lx = gamepad.axis_data(Axis::LeftStickX).map_or(0.0, |d| d.value());
    let raw_ly = gamepad.axis_data(Axis::LeftStickY).map_or(0.0, |d| d.value());
    let raw_rx = gamepad.axis_data(Axis::RightStickX).map_or(0.0, |d| d.value());
    let raw_ry = gamepad.axis_data(Axis::RightStickY).map_or(0.0, |d| d.value());

    // 将原始数据写入 RAW_CONTROLLER_DATA 供校准线程使用
    {
        let mut raw_data = RAW_CONTROLLER_DATA.write().unwrap();
        raw_data.left_stick.x = raw_lx;
        raw_data.left_stick.y = raw_ly;
        raw_data.right_stick.x = raw_rx;
        raw_data.right_stick.y = raw_ry;
    }

    let (final_lx, final_ly, final_rx, final_ry) = get_calibrated_stick_values(raw_lx, raw_ly, raw_rx, raw_ry);

    controller_data.left_stick.x = final_lx;
    controller_data.left_stick.y = final_ly;
    controller_data.right_stick.x = final_rx;
    controller_data.right_stick.y = final_ry;

    // --- 新增: 使用通用函数计算并存储摇杆旋转状态 ---
    controller_data.left_stick_rotation = update_joystick_rotation_state(JoystickSource::LeftStick, final_lx, final_ly);
    controller_data.right_stick_rotation = update_joystick_rotation_state(JoystickSource::RightStick, final_rx, final_ry);

    controller_data.right_stick.is_pressed = gamepad.is_pressed(gilrs::Button::RightThumb);
    controller_data.left_stick.is_pressed = gamepad.is_pressed(gilrs::Button::LeftThumb);

    controller_data.left_trigger.is_pressed = gamepad.is_pressed(gilrs::Button::LeftTrigger2);
    controller_data.right_trigger.is_pressed = gamepad.is_pressed(gilrs::Button::RightTrigger2);
    log::info!(
        "Left Stick Rotation: {:?}, Right Stick Rotation: {:?}",
        controller_data.left_stick_rotation,
        controller_data.right_stick_rotation
    );

    controller_data.left_trigger.value = gamepad.button_data(gilrs::Button::LeftTrigger2)
                                                .map(|data| data.value())
                                                .unwrap_or(0.0);
    controller_data.right_trigger.value = gamepad.button_data(gilrs::Button::RightTrigger2)
                                                 .map(|data| data.value())
                                                 .unwrap_or(0.0);

    pack_and_send_data(&controller_data);
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
            && pid.eq_ignore_ascii_case(device.product_id.as_deref().unwrap())
        {
            _poll_other_controllers(gamepad);
        }
    }
}

/// 根据控制器类型分发轮询任务
fn poll_controller(device: &DeviceInfo) {
    match device.controller_type {
        // Xbox控制器特殊处理
        ControllerType::Xbox => {
            #[cfg(target_os = "windows")]
            {
                // windows下，若 UUID 非法，则特殊处理轮询
                if device.uuid_is_invalid {
                    xbox::poll_xbox_controller(device)
                } else {
                    poll_other_controllers(device)
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                poll_other_controllers(device)
            }
        }
        _ => {
            if device.uuid_is_invalid {
                // TODO：未知控制器处理方法，windows 下拟调用xbox方法，其他平台报错
                #[cfg(target_os = "windows")]
                {
                    xbox::poll_xbox_controller(device)
                }
                #[cfg(not(target_os = "windows"))]
                {
                    log::error!("不受支持的控制器：{device:#?}");
                    disconnect_device();
                }
            } else {
                poll_other_controllers(device)
            }
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
                        log::info!("❌ 设备断开: {}", device.name);
                    }
                    last_device = None;
                }
                _ => (), // 无状态变化
            }

            // 执行设备状态轮询
            if let Some(device) = &last_device {
                poll_controller(device);
                mapping::map(&CONTROLLER_DATA.read().unwrap());
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
        let gilrs = Gilrs::new().expect("Failed to init Gilrs");
        {
            *GLOBAL_GILRS.lock().unwrap() = Some(gilrs);
        }

        loop {
            if let Some(gilrs) = GLOBAL_GILRS.lock().unwrap_or_else(|poisoned| {
                log::warn!("GLOBAL_GILRS 互斥锁已被污染，正在恢复...");
                poisoned.into_inner()
            }).as_mut() {
                // 清空事件队列但不处理
                while let Some(Event { event, id, .. }) = gilrs.next_event_blocking(Some(Duration::from_millis(1))) {
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
            log::info!("尝试连接上次连接的设备: {:?}", last_device);
            let devices = query_devices(app_handle.clone()); // query_devices 现在是同步的
            if let Some(device_info) = devices.into_iter().find(|d| {
                let last_vid_str = format!("{:04x}", last_device.vid);
                let last_pid_str = format!("{:04x}", last_device.pid);
                let last_sub_pid_str = format!("{:04x}", last_device.sub_pid);

                let vid_matches = d.vendor_id == last_vid_str;

                let pid_matches = if last_device.pid == 0 {
                    true
                } else {
                    d.product_id.as_deref().map_or(false, |pid| pid == last_pid_str)
                };

                let sub_pid_matches = if last_device.sub_pid == 0 {
                    true
                } else {
                    d.sub_product_id.as_deref().map_or(false, |sub_pid| sub_pid == last_sub_pid_str)
                };

                // log::debug!("匹配检查: DeviceInfo {:?} vs LastConnectedDevice {:?}", d, last_device);
                // log::debug!("  VID: {} == {} -> {}", d.vendor_id, last_vid_str, vid_matches);
                // log::debug!("  PID: {:?} == {} -> {}", d.product_id, last_pid_str, pid_matches);
                // log::debug!("  SubPID: {:?} == {} -> {}", d.sub_product_id, last_sub_pid_str, sub_pid_matches);
                // log::debug!("  总匹配: {}", vid_matches && (pid_matches || sub_pid_matches));

                vid_matches && (pid_matches || sub_pid_matches)
            }) {
                log::info!("找到匹配的设备，尝试连接: {:?}", device_info);
                if use_device(device_info.name.clone()) { // use_device 现在是同步的
                    log::info!("成功自动连接上次设备");
                    if let Err(e) = app_handle.emit("auto_connect_success", device_info) {
                        log::error!("发送 auto_connect_success 事件失败: {e}");
                    }
                    return;
                } else {
                    log::error!("自动连接上次设备失败");
                }
            } else {
                log::warn!("未找到上次连接的设备: {:?}", last_device);
            }
        } else {
            log::info!("记住上次连接状态已启用，但没有上次连接的设备信息。");
        }
    }
}
