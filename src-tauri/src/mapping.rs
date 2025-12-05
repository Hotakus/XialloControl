#![allow(dead_code)]

// --- 依赖项和常量 ฅ^•ﻌ•^ฅ ---
use crate::controller::datas::{ControllerButtons, ControllerDatas, JoystickRotation};
use crate::controller::{CURRENT_DEVICE, ControllerType};
use crate::{mapping, preset};
use crate::xeno_utils;
use enigo::{Enigo, Keyboard, Mouse};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

// --- 数据结构定义 (•̀ω•́)✧ ---

/// 按键检测模式
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CheckMode {
    Single,
    Double,
    Long,
}

impl Default for CheckMode {
    fn default() -> Self {
        Self::Single
    }
}

/// 按键检测的动态状态
#[derive(Clone, Debug, Default)]
pub struct ButtonCheckState {
    /// 上次按下的时间
    pub last_press_time: Option<Instant>,
    /// 上次释放的时间
    pub last_release_time: Option<Instant>,
    /// 当前是否处于长按触发后的状态
    pub long_press_triggered: bool,
    /// 单击事件是否已准备好触发（在等待双击可能性时）
    pub single_press_pending: bool,
    /// 双击是否已经触发
    pub double_press_triggered: bool,
    /// 记录完整的按下次数
    pub press_count: u32,
    /// 记录完整的释放次数
    pub release_count: u32,
    /// 上一次的按键状态，用于检测状态变化
    pub last_button_state: bool,
    /// 第一次按下的时间，用于双击判断
    pub first_press_time: Option<Instant>,
}

/// 触发状态，用于控制按键的重复触发和加速。
/// 当按键被持续按下时，它会以一定的间隔重复触发，并且间隔会逐渐减小（加速）。
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TriggerState {
    /// 持续触发开关
    continually_trigger: bool,
    /// 当前触发间隔（毫秒）。
    interval: u64,
    /// 初始触发间隔（毫秒），用于重置。
    initial_interval: u64,
    /// 最小触发间隔（毫秒），用于限制加速后的最小间隔。
    min_interval: u64,
    /// 加速因子，每次触发后，间隔会乘以这个值。
    acceleration: f64,

    #[serde(skip)] // 跳过序列化和反序列化
    #[serde(default = "TriggerState::default_instant")] // 反序列化时自动调用
    last_trigger: Instant,

    #[serde(skip)] // 跳过序列化和反序列化
    /// 按键是否被按下的状态，用于非连续触发模式
    is_pressed: bool,
}

impl TriggerState {
    /// 默认的 `Instant::now()` 函数，用于反序列化时初始化。
    fn default_instant() -> Instant {
        Instant::now()
    }

    /// 使用指定的参数创建一个新的 `TriggerState`。
    pub fn new(initial_interval: u64, min_interval: u64, acceleration: f64) -> Self {
        Self {
            continually_trigger: false,
            interval: initial_interval,
            initial_interval,
            min_interval,
            acceleration,
            last_trigger: Instant::now(),
            is_pressed: false,
        }
    }

    /// 检查是否应该触发。如果自上次触发以来经过的时间超过了当前间隔，则返回 `true`。
    /// 触发后，会自动更新下一次的触发间隔（加速）。
    pub fn should_trigger(&mut self, button_is_pressed: bool) -> bool {
        if self.continually_trigger {
            // 连续触发模式：只有在按键被按下且时间间隔满足条件时才触发
            if button_is_pressed && self.last_trigger.elapsed().as_millis() as u64 >= self.interval
            {
                // 触发后加速，更新间隔时间
                self.interval = ((self.interval as f64) * self.acceleration)
                    .max(self.min_interval as f64) as u64;
                self.last_trigger = Instant::now();
                true
            } else {
                false
            }
        } else {
            // 非连续触发模式：检测按键状态变化
            if button_is_pressed != self.is_pressed {
                // 按键状态发生变化，更新状态并返回true触发一次
                self.is_pressed = button_is_pressed;
                true
            } else {
                // 按键状态没有变化，不触发
                false
            }
        }
    }

    /// 重置触发状态到初始间隔。
    pub fn reset(&mut self) {
        self.interval = self.initial_interval;
    }

    /// 获取按键是否被按下的状态
    pub fn is_key_pressed(&self) -> bool {
        self.is_pressed
    }

    /// 设置按键状态
    pub fn set_key_pressed(&mut self, pressed: bool) {
        self.is_pressed = pressed;
    }
}

impl Default for TriggerState {
    fn default() -> Self {
        Self::new(300, 100, 0.8)
    }
}

/// 摇杆映射的动态状态, 用于追踪摇杆的实时数据
#[derive(Clone, Debug, Default)]
pub struct JoystickMappingState {
    accumulated_angle: f32,
    accumulated_value: f32,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum MappingTag {
    Normal,      // 普通映射
    Group,       // 组映射
    Tail,        // 尾部标识
}

/// 映射配置，将一个手柄按钮组合映射到一个键盘或鼠标操作。
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Mapping {
    /// 映射的唯一标识符。
    id: u64,
    /// 组合后的手柄按钮字符串，例如 "A+B"。
    composed_button: String,
    /// 组合后的快捷键字符串，例如 "Ctrl+C"。
    composed_shortcut_key: String,

    /// 按键检测模式。
    #[serde(default)]
    check_mode: CheckMode,
    /// 按键检测模式的参数（例如，长按时间或双击间隔）。
    #[serde(default)]
    check_mode_param: u64,

    // 扳机触发阈值
    trigger_theshold: f32,

    #[serde(flatten)]
    /// 解析后的实际操作指令。
    action: Action,

    /// 触发选项，用于控制按键的重复触发行为。
    #[serde(flatten)]
    trigger_state: TriggerState,

    #[serde(default)]
    mapping_tag: Option<MappingTag>,
}

impl Mapping {
    /// 创建一个带有默认 `Action` 和 `TriggerState` 的新 `Mapping`。
    pub fn new(id: u64, composed_button: String, composed_shortcut_key: String) -> Self {
        Self {
            id,
            composed_button,
            composed_shortcut_key,
            check_mode: CheckMode::default(),
            check_mode_param: 300,
            trigger_theshold: 0.3,
            action: Action::default(),
            trigger_state: TriggerState::default(),
            mapping_tag: None,
        }
    }

    /// 获取映射的唯一 ID。
    pub fn get_id(&self) -> u64 {
        self.id
    }

    /// 获取映射的手柄按钮组合字符串。
    pub fn get_composed_button(&self) -> &str {
        &self.composed_button
    }

    /// 获取映射的快捷键组合字符串。
    pub fn get_composed_key(&self) -> &str {
        &self.composed_shortcut_key
    }
}

/// 包装结构体，用于文件序列化和反序列化。
#[derive(Serialize, Deserialize)]
struct MappingFile {
    mappings: Vec<Mapping>,
}

// --- 操作指令相关定义 (๑>؂<๑)۶ ---

// 定义可执行trait
pub trait Executable {
    fn execute(&self, enigo: &mut Enigo);
    fn execute_press(&self, enigo: &mut Enigo);
    fn execute_release(&self, enigo: &mut Enigo);
}

/// 主要操作类型，代表一个具体的键盘按键、鼠标点击或滚轮事件。
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case", untagged)]
pub enum PrimaryAction {
    /// 按下一个键盘按键。
    KeyPress {
        #[serde(flatten)]
        key: enigo::Key,
    },
    /// 点击一个鼠标按钮。
    MouseClick { button: enigo::Button },
    /// 滚动鼠标滚轮。
    MouseWheel { amount: i32 },
    /// 打开虚拟键盘。
    VirtualKeyboard {
        /// 标记字段，用于序列化和反序列化
        #[serde(default, skip_serializing_if = "Option::is_none")]
        virtual_keyboard: Option<bool>,
    },
    /// 空操作，不执行任何动作。
    None {
        /// 标记字段，用于序列化和反序列化
        #[serde(default, skip_serializing_if = "Option::is_none")]
        none: Option<bool>,
    },
}

impl Executable for PrimaryAction {
    fn execute(&self, enigo: &mut Enigo) {
        match self {
            PrimaryAction::KeyPress { key } => {
                enigo
                    .key(*key, enigo::Direction::Click)
                    .expect("Failed to press key"); // 按下并释放
            }
            PrimaryAction::MouseClick { button } => {
                enigo
                    .button(*button, enigo::Direction::Click)
                    .expect("Failed to clicked mouse button");
            }
            PrimaryAction::MouseWheel { amount } => {
                enigo
                    .scroll(*amount, enigo::Axis::Vertical)
                    .expect("Failed to scroll mouse weight");
            }
            PrimaryAction::VirtualKeyboard { .. } => {
                // 切换虚拟键盘状态（打开/关闭）
                toggle_virtual_keyboard();
            }
            PrimaryAction::None { .. } => {
                // 空操作，不执行任何动作
            }
        }
    }

    fn execute_press(&self, enigo: &mut Enigo) {
        match self {
            PrimaryAction::KeyPress { key } => {
                enigo
                    .key(*key, enigo::Direction::Press)
                    .expect("Failed to press key");
            }
            PrimaryAction::MouseClick { button } => {
                enigo
                    .button(*button, enigo::Direction::Press)
                    .expect("Failed to press mouse button");
            }
            PrimaryAction::MouseWheel { amount } => {
                // 滚轮没有按下和释放的概念，直接执行滚动
                enigo
                    .scroll(*amount, enigo::Axis::Vertical)
                    .expect("Failed to scroll mouse weight");
            }
            PrimaryAction::VirtualKeyboard { .. } => {
                // 虚拟键盘没有按下和释放的概念，直接执行切换操作
                toggle_virtual_keyboard();
            }
            PrimaryAction::None { .. } => {
                // 空操作，不执行任何动作
            }
        }
    }

    fn execute_release(&self, enigo: &mut Enigo) {
        match self {
            PrimaryAction::KeyPress { key } => {
                enigo
                    .key(*key, enigo::Direction::Release)
                    .expect("Failed to release key");
            }
            PrimaryAction::MouseClick { button } => {
                enigo
                    .button(*button, enigo::Direction::Release)
                    .expect("Failed to release mouse button");
            }
            PrimaryAction::MouseWheel { amount: _amount } => {
                // 滚轮没有按下和释放的概念，不做任何操作
            }
            PrimaryAction::VirtualKeyboard { .. } => {
                // 虚拟键盘没有按下和释放的概念，不做任何操作
            }
            PrimaryAction::None { .. } => {
                // 空操作，不执行任何动作
            }
        }
    }
}

/// 完整的操作指令，包含修饰键和主要操作。
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Action {
    /// 需要按下的修饰键列表，如 `[Key::Shift, Key::Control]`。
    pub modifiers: Vec<enigo::Key>,

    #[serde(flatten)]
    /// 最终执行的核心动作。
    pub primary: PrimaryAction,
}

impl Default for Action {
    fn default() -> Self {
        Self {
            modifiers: vec![],
            primary: PrimaryAction::KeyPress {
                key: enigo::Key::Space,
            },
        }
    }
}

impl Executable for Action {
    fn execute(&self, enigo: &mut Enigo) {
        // 1. 按下所有修饰键
        for modifier in &self.modifiers {
            enigo
                .key(*modifier, enigo::Direction::Press)
                .expect("Failed to press modifier key");
        }

        // 2. 执行主操作
        self.primary.execute(enigo);

        // 3. 释放所有修饰键 (以相反顺序)
        for modifier in self.modifiers.iter().rev() {
            enigo
                .key(*modifier, enigo::Direction::Release)
                .expect("Failed to release modifier key");
        }
    }

    fn execute_press(&self, enigo: &mut Enigo) {
        // 1. 按下所有修饰键
        for modifier in &self.modifiers {
            enigo
                .key(*modifier, enigo::Direction::Press)
                .expect("Failed to press modifier key");
        }

        // 2. 执行主操作的按下
        self.primary.execute_press(enigo);
    }

    fn execute_release(&self, enigo: &mut Enigo) {
        // 1. 执行主操作的释放
        self.primary.execute_release(enigo);

        // 2. 释放所有修饰键 (以相反顺序)
        for modifier in self.modifiers.iter().rev() {
            enigo
                .key(*modifier, enigo::Direction::Release)
                .expect("Failed to release modifier key");
        }
    }
}

// --- 错误处理和辅助类型 (•ω•) ---

/// 解析错误类型。
#[derive(Debug)]
pub enum ParseError {
    /// 没有找到主操作。
    NoPrimaryAction,
    /// 找到了多个主操作。
    MultiplePrimaryActions,
    /// 无法识别的按键字符串。
    UnknownKey(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::NoPrimaryAction => write!(f, "没有找到主操作"),
            ParseError::MultiplePrimaryActions => write!(f, "找到了多个主操作"),
            ParseError::UnknownKey(key) => write!(f, "无法识别的按键字符串: {}", key),
        }
    }
}

/// 映射更新配置，用于配置对象模式重构 update_mapping 函数。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MappingUpdateConfig {
    pub id: u64,
    pub composed_button: Option<String>,
    pub composed_shortcut_key: Option<String>,
    pub trigger_state: Option<TriggerState>,
    pub trigger_theshold: Option<f32>,
    pub amount: Option<i32>,
    pub check_mode: Option<CheckMode>,
    pub check_mode_param: Option<u64>,
    pub mapping_tag: Option<MappingTag>,
}

impl MappingUpdateConfig {
    /// 创建一个新的配置对象，只需要提供必需的 id 参数
    pub fn new(id: u64) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    /// 设置组合按钮字符串
    pub fn with_composed_button(mut self, button: String) -> Self {
        self.composed_button = Some(button);
        self
    }

    /// 设置组合快捷键字符串
    pub fn with_composed_shortcut_key(mut self, key: String) -> Self {
        self.composed_shortcut_key = Some(key);
        self
    }

    /// 设置触发状态
    pub fn with_trigger_state(mut self, state: TriggerState) -> Self {
        self.trigger_state = Some(state);
        self
    }

    /// 设置触发阈值
    pub fn with_trigger_threshold(mut self, threshold: f32) -> Self {
        self.trigger_theshold = Some(threshold);
        self
    }

    /// 设置滚轮量
    pub fn with_amount(mut self, amount: i32) -> Self {
        self.amount = Some(amount);
        self
    }

    /// 设置检测模式
    pub fn with_check_mode(mut self, mode: CheckMode) -> Self {
        self.check_mode = Some(mode);
        self
    }

    /// 设置检测模式参数
    pub fn with_check_mode_param(mut self, param: u64) -> Self {
        self.check_mode_param = Some(param);
        self
    }
}

/// Enigo 命令类型，用于工作线程间通信。
#[derive(Debug)]
pub enum EnigoCommand {
    /// 执行一个 `Action`。
    Execute(Action),
    /// 执行一个 `Action` 的按下操作。
    ExecutePress(Action),
    /// 执行一个 `Action` 的释放操作。
    ExecuteRelease(Action),
}

// --- 全局静态变量 (づ￣ 3￣)づ ---

/// 全局映射配置缓存，使用 `RwLock` 确保线程安全。
pub static GLOBAL_MAPPING_CACHE: Lazy<RwLock<Vec<Mapping>>> = Lazy::new(|| {
    let mappings = vec![];
    RwLock::new(mappings)
});

/// 副预设映射配置缓存
pub static SUB_MAPPING_CACHE: Lazy<RwLock<Vec<Mapping>>> = Lazy::new(|| RwLock::new(vec![]));

/// TOML 配置文件名。
const DEFAULT_MAPPINGS_FILE: &str = "mappings.toml";
pub static MAPPING_FILE_PATH: Lazy<RwLock<PathBuf>> =
    Lazy::new(|| RwLock::from(PathBuf::from(DEFAULT_MAPPINGS_FILE)));

/// 全局手柄按键布局映射，例如将 "Y" 映射到 `ControllerButtons::North`。
/// 存储不同类型手柄的布局。
type ButtonLayout = HashMap<&'static str, ControllerButtons>;
type ControllerLayouts = HashMap<ControllerType, Arc<ButtonLayout>>;
pub static CONTROLLER_LAYOUT_MAP: Lazy<RwLock<ControllerLayouts>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// 动态触发状态，存储每个映射的触发状态。
pub static DYNAMIC_TRIGGER_STATES: Lazy<RwLock<HashMap<u64, TriggerState>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// 动态摇杆映射状态, 存储每个摇杆映射的实时数据.
pub static JOYSTICK_MAPPING_STATES: Lazy<RwLock<HashMap<u64, JoystickMappingState>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// 全局变量来跟踪虚拟键盘进程
static VIRTUAL_KEYBOARD_PROCESS: Lazy<Mutex<Option<std::process::Child>>> =
    Lazy::new(|| Mutex::new(None));

/// 动态按键检测状态，存储每个映射的按键检测状态。
pub static BUTTON_CHECK_STATES: Lazy<RwLock<HashMap<u64, ButtonCheckState>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Enigo 工作线程的发送器，用于向其发送执行命令。
pub static ENIGO_SENDER: Lazy<Sender<EnigoCommand>> = Lazy::new(|| {
    let (tx, rx): (Sender<EnigoCommand>, Receiver<EnigoCommand>) = channel();
    // 启动工作线程来处理 Enigo 命令
    thread::spawn(move || enigo_worker(rx));
    tx
});


pub fn toggle_virtual_keyboard() {
    let mut lock = VIRTUAL_KEYBOARD_PROCESS.lock().unwrap();
    if let Some(child) = lock.as_mut() {
        // 如果进程存在，尝试关闭它
        if let Err(e) = child.kill() {
            log::error!("关闭虚拟键盘失败: {e}");
        } else {
            log::info!("虚拟键盘已关闭");
            *lock = None; // 清除进程句柄
        }
    } else {
        // 启动 TabTip.exe 作为虚拟键盘
        match Command::new("osk.exe")
            .spawn()
        {
            Ok(child) => {
                log::info!("虚拟键盘已启动");
                *lock = Some(child); // 保存进程句柄
            }
            Err(e) => {
                log::error!("启动虚拟键盘失败: {e}");
            }
        }
    }
}

// --- 按键检测核心逻辑 (ﾉ>ω<)ﾉ ---

/// 检查按键状态并根据检测模式返回是否应该触发
///
/// # 参数
/// * `button_is_pressed` - 按键当前是否被按下
/// * `check_mode` - 检测模式（单击、双击、长按）
/// * `check_mode_param` - 检测模式参数（双击间隔或长按时间）
/// * `check_state` - 按键检测状态
///
/// # 返回值
/// 返回一个布尔值，表示是否应该触发映射动作
pub fn check_button_press(
    button_is_pressed: bool,
    check_mode: CheckMode,
    check_mode_param: u64,
    check_state: &mut ButtonCheckState,
) -> bool {
    let now = Instant::now();

    match check_mode {
        CheckMode::Single => {
            // 单击模式：按键按下时触发
            if button_is_pressed {
                // 检查是否是第一次按下，或者从释放状态再次按下
                if check_state.last_press_time.is_none() {
                    // 第一次按下，直接触发
                    check_state.last_press_time = Some(now);
                    check_state.last_release_time = None;
                    return true;
                } else if check_state.last_release_time.is_some() {
                    // 从释放状态再次按下，检查时间间隔防止误触
                    let time_since_release = now
                        .duration_since(check_state.last_release_time.unwrap())
                        .as_millis() as u64;
                    if time_since_release > 50 {
                        check_state.last_press_time = Some(now);
                        check_state.last_release_time = None;
                        return true;
                    }
                }
            } else if !button_is_pressed && check_state.last_press_time.is_some() {
                // 按键释放，记录释放时间
                check_state.last_release_time = Some(now);
            }

            false
        }

        CheckMode::Double => {
            let now = Instant::now();

            // 检测按键状态变化
            let state_changed = button_is_pressed != check_state.last_button_state;

            // 只有在状态真正发生变化时才进行处理
            if state_changed {
                if button_is_pressed {
                    // 从释放变为按下 - 记录一次按下
                    check_state.press_count += 1;
                    check_state.last_press_time = Some(now);

                    // 如果是第一次按下，记录第一次按下时间
                    if check_state.press_count == 1 {
                        check_state.first_press_time = Some(now);
                    }

                    // 检查是否满足双击条件（第二次按下时）
                    if check_state.press_count >= 2 {
                        if let Some(first_press_time) = check_state.first_press_time {
                            let time_since_first_press =
                                now.duration_since(first_press_time).as_millis() as u64;

                            // 检查时间窗口
                            if time_since_first_press <= check_mode_param {
                                // 检查是否已经触发过双击（防止重复触发）
                                if !check_state.double_press_triggered {
                                    // 触发双击
                                    check_state.double_press_triggered = true;
                                    return true;
                                }
                            } else {
                                // 超时，重置计数
                                reset_double_click_state(check_state);
                            }
                        }
                    }
                } else {
                    // 从按下变为释放 - 记录一次释放
                    check_state.release_count += 1;
                    check_state.last_release_time = Some(now);
                }

                // 更新上一次的按键状态
                check_state.last_button_state = button_is_pressed;
            } else if button_is_pressed {
                // 状态没有变化但按键仍被按下
                // 检查是否超时，超时则重置状态
                if let Some(first_press_time) = check_state.first_press_time {
                    let time_since_first_press =
                        now.duration_since(first_press_time).as_millis() as u64;
                    if time_since_first_press > check_mode_param {
                        // 长按超时，重置状态
                        reset_double_click_state(check_state);
                    }
                }
            }

            // 检查是否需要重置状态（超时或完成双击）
            if should_reset_state(check_state, now, check_mode_param) {
                reset_double_click_state(check_state);
            }

            false
        }

        CheckMode::Long => {
            // 长按模式：按键按下超过指定时间后触发
            if button_is_pressed {
                if check_state.last_press_time.is_none() {
                    // 开始按下
                    check_state.last_press_time = Some(now);
                    check_state.long_press_triggered = false;
                } else {
                    // 持续按下中
                    let press_duration = now
                        .duration_since(check_state.last_press_time.unwrap())
                        .as_millis() as u64;
                    if press_duration >= check_mode_param && !check_state.long_press_triggered {
                        // 达到长按时间，触发一次
                        check_state.long_press_triggered = true;
                        return true;
                    } else if press_duration >= check_mode_param && check_state.long_press_triggered
                    {
                        // 长按已经触发过，持续按下中，继续触发
                        return true;
                    }
                }
            } else {
                // 按键释放，重置状态
                check_state.last_press_time = None;
                check_state.long_press_triggered = false;
            }

            false
        }
    }
}

/// 重置双击状态的辅助函数
fn reset_double_click_state(check_state: &mut ButtonCheckState) {
    check_state.press_count = 0;
    check_state.release_count = 0;
    check_state.double_press_triggered = false;
    check_state.first_press_time = None;
    check_state.single_press_pending = false;
}

/// 判断是否需要重置双击状态的辅助函数
fn should_reset_state(check_state: &ButtonCheckState, now: Instant, timeout: u64) -> bool {
    if let Some(first_press_time) = check_state.first_press_time {
        let time_since_first_press = now.duration_since(first_press_time).as_millis() as u64;
        // 超时或者已经完成双击
        time_since_first_press > timeout
            || (check_state.press_count >= 2
                && check_state.release_count >= 2
                && check_state.double_press_triggered)
    } else {
        false
    }
}

// --- 虚拟键盘相关函数 (ﾉ>ω<)ﾉ ---


// --- 核心逻辑函数 (ﾉ>ω<)ﾉ ---

/// 设置相对 全局配置目录 的映射配置文件路径。
pub fn set_mapping_file_path(path: PathBuf) {
    let mut lock = MAPPING_FILE_PATH.write().unwrap();
    *lock = path;
}

/// 设置相对 全局配置目录 的映射配置文件路径。
pub fn get_mapping_file_path() -> PathBuf {
    MAPPING_FILE_PATH.read().unwrap().clone()
}

/// 内部加载映射配置的实现，从文件读取。
fn load_mappings_internal() -> Vec<Mapping> {
    let mappings_path = xeno_utils::get_config_path(get_mapping_file_path().to_str().unwrap());
    load_mappings_from_path(mappings_path)
}

pub fn load_mappings_from_path(path: PathBuf) -> Vec<Mapping> {
    if !path.exists() {
        log::warn!("映射配置文件不存在，将创建空文件: {path:#?}");
        // 创建空映射文件
        let mapping_file = MappingFile { mappings: vec![] };
        if let Err(e) = xeno_utils::write_toml_file(&path, &mapping_file) {
            log::error!("创建空映射文件失败: {e}");
        }
        return vec![];
    }

    match xeno_utils::read_toml_file::<MappingFile>(&path) {
        Ok(mapping_file) => {
            log::info!("成功加载 {} 条映射配置", mapping_file.mappings.len());
            mapping_file.mappings
        }
        Err(e) => {
            log::error!("加载映射配置失败: {e}");
            vec![]
        }
    }
}

/// 将指定路径的映射加载到副预设缓存中
pub fn load_sub_mappings(path: PathBuf) {
    let mut cache = SUB_MAPPING_CACHE.write().unwrap();
    let config_path = xeno_utils::get_config_path(path.to_str().unwrap());
    *cache = load_mappings_from_path(config_path);
}

/// 将映射配置加载到全局缓存中。
pub fn load_mappings() -> Vec<Mapping> {
    let mut cache = GLOBAL_MAPPING_CACHE.write().unwrap();
    *cache = load_mappings_internal();
    cache.clone()
}

#[tauri::command]
pub fn refresh_mappings() -> Vec<Mapping> {
    load_mappings()
}

/// 将全局映射缓存保存到文件中。
pub fn save_mappings() {
    // 确保配置目录存在
    xeno_utils::ensure_config_dir();

    let mappings = get_mappings_internal();
    let mappings_path = xeno_utils::get_config_path(get_mapping_file_path().to_str().unwrap());

    let mapping_file = MappingFile {
        mappings: mappings.clone(),
    };

    match xeno_utils::write_toml_file(&mappings_path, &mapping_file) {
        Ok(_) => log::info!("映射配置已保存到: {mappings_path:#?}",),
        Err(e) => log::error!("保存映射配置失败: {e:#?} {mappings:#?}"),
    }
}

/// 获取当前映射配置的内部实现（线程安全）。
pub fn get_mappings_internal() -> Vec<Mapping> {
    GLOBAL_MAPPING_CACHE.read().unwrap().clone()
}

/// Tauri 命令：设置所有映射配置。
#[tauri::command]
pub fn set_mapping(mapping: Vec<Mapping>) {
    log::debug!("更新映射配置: {mapping:#?}");
    {
        let mut cache = GLOBAL_MAPPING_CACHE.write().unwrap();
        *cache = mapping;
    }
    // 更新后立即保存
    save_mappings();
    log::debug!("映射缓存已更新并保存");
}

/// 将映射配置保存到指定文件路径（不影响全局状态）
///
/// # 参数
/// * `mappings` - 要保存的映射列表
/// * `file_path` - 目标文件路径
///
/// # 返回值
/// * `Result<(), String>` - 成功返回 Ok(())，失败返回错误信息
pub fn save_mappings_to_file(mappings: Vec<Mapping>, file_path: PathBuf) -> Result<(), String> {
    // 确保配置目录存在
    let config_path = xeno_utils::get_config_path(file_path.to_str().unwrap());
    xeno_utils::ensure_config_dir();

    let mapping_file = MappingFile {
        mappings: mappings.to_vec(),
    };

    match xeno_utils::write_toml_file(&config_path, &mapping_file) {
        Ok(_) => {
            log::info!("映射配置已保存到: {config_path:#?}");
            Ok(())
        }
        Err(e) => {
            log::error!("保存映射配置失败: {e:#?}");
            Err(format!("保存映射配置失败: {e}"))
        }
    }
}

/// 从 MappingUpdateConfig 创建 Mapping 对象（不添加到全局缓存）
/// 这个函数复用了 add_mapping 中的映射创建逻辑
///
/// # 参数
/// * `config` - 映射更新配置
///
/// # 返回值
/// * `Result<Mapping, String>` - 成功返回创建的 Mapping 对象，失败返回错误信息
pub fn create_mapping_from_config(config: MappingUpdateConfig) -> Result<Mapping, String> {
    // 对于所有映射类型，我们都从 composed_shortcut_key 解析出 Action
    if let Some(composed_shortcut_key) = &config.composed_shortcut_key {
        match parse_composed_key_to_action(composed_shortcut_key) {
            Ok(mut action) => {
                // 如果是滚轮动作且自定义了 amount, 则覆盖
                if let (PrimaryAction::MouseWheel { .. }, Some(new_amount)) =
                    (&mut action.primary, config.amount)
                {
                    action.primary = PrimaryAction::MouseWheel { amount: new_amount };
                }

                let id = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                let trigger_state = config.trigger_state.unwrap_or_default();
                let new_mapping = Mapping {
                    id,
                    composed_button: config.composed_button.unwrap_or_default(),
                    composed_shortcut_key: composed_shortcut_key.clone(),
                    check_mode: config.check_mode.unwrap_or_default(),
                    check_mode_param: config.check_mode_param.unwrap_or(300),
                    trigger_theshold: config.trigger_theshold.unwrap_or(0.3),
                    action,
                    trigger_state: trigger_state.clone(),
                    mapping_tag: None,
                };

                Ok(new_mapping)
            }
            Err(e) => {
                log::error!("解析快捷键/动作失败 '{composed_shortcut_key}': {e:?}");
                Err(format!("解析快捷键/动作失败: {e}"))
            }
        }
    } else {
        log::error!("创建映射失败，缺少 composed_shortcut_key");
        Err("创建映射失败，缺少 composed_shortcut_key".to_string())
    }
}

/// 创建空的映射文件
///
/// # 参数
/// * `file_path` - 目标文件路径
///
/// # 返回值
/// * `Result<(), String>` - 成功返回 Ok(())，失败返回错误信息
pub fn create_empty_mapping_file(file_path: PathBuf) -> Result<(), String> {
    // 确保配置目录存在
    let config_path = xeno_utils::get_config_path(file_path.to_str().unwrap());
    xeno_utils::ensure_config_dir();

    // 创建空映射文件
    let mapping_file = MappingFile {
        mappings: vec![],
    };

    match xeno_utils::write_toml_file(&config_path, &mapping_file) {
        Ok(_) => {
            log::info!("空映射文件已创建: {config_path:#?}");
            Ok(())
        }
        Err(e) => {
            log::error!("创建空映射文件失败: {e:#?}");
            Err(format!("创建空映射文件失败: {e}"))
        }
    }
}

/// Tauri 命令：更新一个已存在的映射配置。
#[tauri::command]
pub fn update_mapping(config: MappingUpdateConfig) -> bool {
    log::debug!("请求更新映射配置: {config:#?}");
    let id = config.id;
    let mut cache = GLOBAL_MAPPING_CACHE.write().unwrap();
    if let Some(mapping) = cache.iter_mut().find(|m| m.id == id) {
        // 如果提供了新的组合快捷键字符串，则解析它
        if let Some(composed_shortcut_key) = &config.composed_shortcut_key {
            match parse_composed_key_to_action(composed_shortcut_key) {
                Ok(mut action) => {
                    // 如果是滚轮动作且自定义了 amount, 则覆盖
                    if let (PrimaryAction::MouseWheel { .. }, Some(new_amount)) =
                        (&mut action.primary, config.amount)
                    {
                        action.primary = PrimaryAction::MouseWheel { amount: new_amount };
                    }

                    // 更新映射的字段，只更新提供的字段
                    if let Some(button) = &config.composed_button {
                        mapping.composed_button = button.clone();
                    }
                    mapping.composed_shortcut_key = composed_shortcut_key.clone();
                    if let Some(threshold) = config.trigger_theshold {
                        mapping.trigger_theshold = threshold;
                    }
                    if let Some(mode) = config.check_mode {
                        mapping.check_mode = mode;
                    }
                    if let Some(param) = config.check_mode_param {
                        mapping.check_mode_param = param;
                    }
                    if let Some(tag) = &config.mapping_tag {
                        mapping.mapping_tag = Some(tag.clone());
                    }
                    mapping.action = action;

                    // 同步更新 DYNAMIC_TRIGGER_STATES 中的触发状态
                    if let Some(trigger_state) = &config.trigger_state {
                        // 更新 mapping 对象本身的 trigger_state 字段
                        mapping.trigger_state.continually_trigger =
                            trigger_state.continually_trigger;
                        mapping.trigger_state.interval = trigger_state.interval;
                        mapping.trigger_state.initial_interval = trigger_state.initial_interval;
                        mapping.trigger_state.min_interval = trigger_state.min_interval;
                        mapping.trigger_state.acceleration = trigger_state.acceleration;
                        // 注意：不更新 last_trigger 和 is_pressed，保持原有的状态

                        let mut trigger_states = DYNAMIC_TRIGGER_STATES.write().unwrap();
                        if let Some(existing_trigger_state) = trigger_states.get_mut(&id) {
                            // 更新现有的触发状态
                            existing_trigger_state.continually_trigger = trigger_state.continually_trigger;
                            existing_trigger_state.interval = trigger_state.interval;
                            existing_trigger_state.initial_interval = trigger_state.initial_interval;
                            existing_trigger_state.min_interval = trigger_state.min_interval;
                            existing_trigger_state.acceleration = trigger_state.acceleration;
                            // 注意：不更新 last_trigger，保持原有的时间状态
                        } else {
                            // 如果不存在，则插入新的触发状态
                            trigger_states.insert(id, trigger_state.clone());
                        }
                        drop(trigger_states);

                        // 同步更新 BUTTON_CHECK_STATES 中的按键检测状态
                        let mut button_check_states = BUTTON_CHECK_STATES.write().unwrap();
                        // 当检测模式或参数发生变化时，重置按键检测状态
                        if let Some(check_state) = button_check_states.get_mut(&id) {
                            // 如果检测模式改变，重置整个状态
                            if config.check_mode.is_some()
                                && mapping.check_mode != config.check_mode.unwrap_or_default()
                            {
                                *check_state = ButtonCheckState::default();
                            }
                            // 如果检测模式参数改变，也重置状态
                            if config.check_mode_param.is_some()
                                && mapping.check_mode_param
                                    != config.check_mode_param.unwrap_or(300)
                            {
                                *check_state = ButtonCheckState::default();
                            }
                        } else {
                            // 如果不存在，则插入新的按键检测状态
                            button_check_states.insert(id, ButtonCheckState::default());
                        }
                        drop(button_check_states);
                    }

                    log::error!("{:#?}", DYNAMIC_TRIGGER_STATES.read().unwrap());
                }
                Err(e) => {
                    log::error!("解析快捷键/动作失败 '{composed_shortcut_key}': {e:?}");
                    return false;
                }
            }
        }
        drop(cache);
        save_mappings();
        return true;
    }
    log::error!("更新失败，未找到 id {id} 的映射");
    false
}

/// Tauri 命令：添加一个新的映射配置。
#[tauri::command]
pub fn add_mapping(config: MappingUpdateConfig) -> bool {
    log::debug!("请求添加映射配置");

    // 对于所有映射类型，我们都从 composed_shortcut_key 解析出 Action
    if let Some(composed_shortcut_key) = &config.composed_shortcut_key {
        match parse_composed_key_to_action(composed_shortcut_key) {
            Ok(mut action) => {
                // 如果是滚轮动作且自定义了 amount, 则覆盖
                if let (PrimaryAction::MouseWheel { .. }, Some(new_amount)) =
                    (&mut action.primary, config.amount)
                {
                    action.primary = PrimaryAction::MouseWheel { amount: new_amount };
                }

                let mut cache = GLOBAL_MAPPING_CACHE.write().unwrap();
                let id = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                let trigger_state = config.trigger_state.unwrap_or_default();
                let new_mapping = Mapping {
                    id,
                    composed_button: config.composed_button.unwrap_or_default(),
                    composed_shortcut_key: composed_shortcut_key.clone(),
                    check_mode: config.check_mode.unwrap_or_default(),
                    check_mode_param: config.check_mode_param.unwrap_or(300),
                    trigger_theshold: config.trigger_theshold.unwrap_or(0.3),
                    action,
                    trigger_state: trigger_state.clone(),
                    mapping_tag: config.mapping_tag.clone(),
                };

                cache.push(new_mapping);
                drop(cache);

                // 同步更新 DYNAMIC_TRIGGER_STATES，添加新的触发状态
                let mut dynamic_trigger_states = DYNAMIC_TRIGGER_STATES.write().unwrap();
                dynamic_trigger_states.insert(id, trigger_state);
                drop(dynamic_trigger_states);

                // 同步更新 BUTTON_CHECK_STATES，添加新的按键检测状态
                let mut button_check_states = BUTTON_CHECK_STATES.write().unwrap();
                button_check_states.insert(id, ButtonCheckState::default());
                drop(button_check_states);

                save_mappings();
                true
            }
            Err(e) => {
                log::error!("解析快捷键/动作失败 '{composed_shortcut_key}': {e:?}");
                false
            }
        }
    } else {
        log::error!("添加映射失败，缺少 composed_shortcut_key");
        false
    }
}

/// Tauri 命令：根据 ID 删除一个映射配置。
#[tauri::command]
pub async fn delete_mapping(id: u64) -> bool {
    log::debug!("请求删除 id {id} 的映射配置");
    let mut cache = GLOBAL_MAPPING_CACHE.write().unwrap();
    let initial_len = cache.len();

    // 使用 retain 方法高效地移除指定 id 的项
    cache.retain(|m| m.get_id() != id);

    let deleted = cache.len() < initial_len;

    if deleted {
        drop(cache);

        // 同步更新 DYNAMIC_TRIGGER_STATES，删除对应的触发状态
        let mut trigger_states = DYNAMIC_TRIGGER_STATES.write().unwrap();
        trigger_states.remove(&id);
        drop(trigger_states);

        // 同步更新 BUTTON_CHECK_STATES，删除对应的按键检测状态
        let mut button_check_states = BUTTON_CHECK_STATES.write().unwrap();
        button_check_states.remove(&id);
        drop(button_check_states);

        save_mappings();
        log::info!("已成功删除 id {id} 的映射");
    } else {
        log::warn!("尝试删除一个不存在的映射 id: {id}");
    }

    deleted
}

/// Tauri 命令：显式请求保存映射配置。
#[tauri::command]
pub fn save_mapping_config() {
    log::debug!("前端请求保存映射配置");
    save_mappings();
}

/// Tauri 命令：更新映射的顺序。
#[tauri::command]
pub fn update_mappings_order(mappings: Vec<Mapping>) -> bool {
    log::debug!("请求更新映射顺序，共 {} 条映射", mappings.len());
    let mut cache = GLOBAL_MAPPING_CACHE.write().unwrap();

    // 直接用前端发送过来的新顺序覆盖整个缓存
    *cache = mappings;

    drop(cache);
    save_mappings();
    true
}

/// Tauri 命令：获取当前所有映射配置。
#[tauri::command]
pub fn get_mappings() -> Vec<Mapping> {
    get_mappings_internal()
}

/// Tauri 命令：根据 ID 获取单个映射配置。
#[tauri::command]
pub fn get_mapping_by_id(id: u64) -> Option<Mapping> {
    let cache = GLOBAL_MAPPING_CACHE.read().unwrap();
    cache.iter().find(|m| m.id == id).cloned()
}

/// 创建 Xbox 手柄的按键布局映射。
fn create_xbox_layout_map() -> HashMap<&'static str, ControllerButtons> {
    let mut xbox_map = HashMap::new();
    xbox_map.insert("Y", ControllerButtons::North);
    xbox_map.insert("X", ControllerButtons::West);
    xbox_map.insert("A", ControllerButtons::South);
    xbox_map.insert("B", ControllerButtons::East);
    xbox_map.insert("RB", ControllerButtons::RB);
    xbox_map.insert("LB", ControllerButtons::LB);
    xbox_map.insert("RT", ControllerButtons::RT);
    xbox_map.insert("LT", ControllerButtons::LT);
    xbox_map.insert("LeftStick", ControllerButtons::LStick);
    xbox_map.insert("RightStick", ControllerButtons::RStick);
    xbox_map.insert("Back", ControllerButtons::Back);
    xbox_map.insert("Start", ControllerButtons::Start);
    xbox_map.insert("Guide", ControllerButtons::Guide);
    xbox_map.insert("DPadUp", ControllerButtons::Up);
    xbox_map.insert("DPadDown", ControllerButtons::Down);
    xbox_map.insert("DPadLeft", ControllerButtons::Left);
    xbox_map.insert("DPadRight", ControllerButtons::Right);
    xbox_map
}

/// 创建 PlayStation 手柄的按键布局映射。
fn create_playstation_layout_map() -> HashMap<&'static str, ControllerButtons> {
    let mut ps_map = HashMap::new();
    ps_map.insert("Triangle", ControllerButtons::North);
    ps_map.insert("Square", ControllerButtons::West);
    ps_map.insert("Cross", ControllerButtons::South);
    ps_map.insert("Circle", ControllerButtons::East);
    ps_map.insert("R1", ControllerButtons::RB);
    ps_map.insert("L1", ControllerButtons::LB);
    ps_map.insert("RT", ControllerButtons::RT);
    ps_map.insert("LT", ControllerButtons::LT);
    ps_map.insert("LeftStick", ControllerButtons::LStick);
    ps_map.insert("RightStick", ControllerButtons::RStick);
    ps_map.insert("Share", ControllerButtons::Back); // PlayStation 的 Share 键通常对应 Xbox 的 Back 键
    ps_map.insert("Options", ControllerButtons::Start); // PlayStation 的 Options 键通常对应 Xbox 的 Start 键
    ps_map.insert("PS", ControllerButtons::Guide); // PlayStation 的 PS 键通常对应 Xbox 的 Guide 键
    ps_map.insert("DPadUp", ControllerButtons::Up);
    ps_map.insert("DPadDown", ControllerButtons::Down);
    ps_map.insert("DPadLeft", ControllerButtons::Left);
    ps_map.insert("DPadRight", ControllerButtons::Right);
    ps_map
}

/// 创建通用手柄的按键布局映射。
fn create_other_layout_map() -> HashMap<&'static str, ControllerButtons> {
    let mut other_map = HashMap::new();
    other_map.insert("Y", ControllerButtons::North);
    other_map.insert("X", ControllerButtons::West);
    other_map.insert("A", ControllerButtons::South);
    other_map.insert("B", ControllerButtons::East);
    other_map.insert("RB", ControllerButtons::RB);
    other_map.insert("LB", ControllerButtons::LB);
    other_map.insert("LeftStick", ControllerButtons::LStick);
    other_map.insert("RightStick", ControllerButtons::RStick);
    other_map.insert("Back", ControllerButtons::Back);
    other_map.insert("Start", ControllerButtons::Start);
    other_map.insert("Guide", ControllerButtons::Guide);
    other_map.insert("DPadUp", ControllerButtons::Up);
    other_map.insert("DPadDown", ControllerButtons::Down);
    other_map.insert("DPadLeft", ControllerButtons::Left);
    other_map.insert("DPadRight", ControllerButtons::Right);
    other_map
}

/// 创建 Nintendo Switch 手柄的按键布局映射。
fn create_switch_layout_map() -> HashMap<&'static str, ControllerButtons> {
    let mut switch_map = HashMap::new();
    switch_map.insert("Y", ControllerButtons::North);
    switch_map.insert("X", ControllerButtons::West);
    switch_map.insert("B", ControllerButtons::South); // Switch 的 B 对应 Xbox 的 A
    switch_map.insert("A", ControllerButtons::East); // Switch 的 A 对应 Xbox 的 B
    switch_map.insert("R", ControllerButtons::RB); // Switch 的 R 对应 Xbox 的 RB
    switch_map.insert("L", ControllerButtons::LB); // Switch 的 L 对应 Xbox 的 LB
    switch_map.insert("RT", ControllerButtons::RT);
    switch_map.insert("LT", ControllerButtons::LT);
    switch_map.insert("LeftStick", ControllerButtons::LStick);
    switch_map.insert("RightStick", ControllerButtons::RStick);
    switch_map.insert("Minus", ControllerButtons::Back); // Switch 的 Minus 对应 Xbox 的 Back
    switch_map.insert("Plus", ControllerButtons::Start); // Switch 的 Plus 对应 Xbox 的 Start
    switch_map.insert("Home", ControllerButtons::Guide); // Switch 的 Home 对应 Xbox 的 Guide
    switch_map.insert("DPadUp", ControllerButtons::Up);
    switch_map.insert("DPadDown", ControllerButtons::Down);
    switch_map.insert("DPadLeft", ControllerButtons::Left);
    switch_map.insert("DPadRight", ControllerButtons::Right);
    switch_map
}

/// 初始化所有支持的手柄按键布局映射。
fn init_controller_layout_maps() {
    let mut map = CONTROLLER_LAYOUT_MAP.write().unwrap();
    if map.is_empty() {
        map.insert(ControllerType::Xbox, Arc::new(create_xbox_layout_map()));
        map.insert(
            ControllerType::PlayStation,
            Arc::new(create_playstation_layout_map()),
        );
        map.insert(ControllerType::Switch, Arc::new(create_switch_layout_map())); // 添加 Switch 布局
        map.insert(ControllerType::Other, Arc::new(create_other_layout_map()));
    }
}

/// 获取当前连接手柄的按键布局映射的只读引用。
pub fn get_current_controller_layout_map() -> Arc<HashMap<&'static str, ControllerButtons>> {
    init_controller_layout_maps();
    let controller_type = CURRENT_DEVICE.read().unwrap().controller_type;
    let map_guard = CONTROLLER_LAYOUT_MAP.read().unwrap();
    map_guard
        .get(&controller_type)
        .unwrap_or_else(|| {
            // log::warn!("未找到 {controller_type:?} 对应的布局，使用通用布局");
            map_guard.get(&ControllerType::Other).unwrap()
        })
        .clone()
}

/// 辅助函数，确保只有一个主操作被设置，防止按键组合解析错误。
fn set_primary(
    primary_field: &mut Option<PrimaryAction>,
    action: PrimaryAction,
) -> Result<(), ParseError> {
    if primary_field.is_some() {
        Err(ParseError::MultiplePrimaryActions)
    } else {
        *primary_field = Some(action);
        Ok(())
    }
}

/// 将数字转换为对应的功能键
fn get_function_key(num: u8) -> enigo::Key {
    // 利用 Rust 的枚举值在内存中是连续的特性
    // 通过 unsafe 代码进行指针运算来获取对应的枚举值
    unsafe {
        let base_key = enigo::Key::F1;
        let base_ptr = &base_key as *const enigo::Key as *const u8;
        let offset_ptr = base_ptr.add((num - 1) as usize);
        let target_ptr = offset_ptr as *const enigo::Key;
        *target_ptr
    }
}

/// 解析按键组合字符串，生成结构化的 `Action`。
/// 例如 "Ctrl+Alt+A" 会被解析成一个带有 `[Control, Alt]` 修饰键和 `KeyPress { key: 'A' }` 主操作的 `Action`。
fn parse_composed_key_to_action(composed: &str) -> Result<Action, ParseError> {
    let mut modifiers = Vec::new();
    let mut primary_action = None;

    for part in composed.split('+').map(|s| {
        if s.len() > 1 {
            s.trim().to_string()
        } else {
            s.to_owned()
        }
    }) {
        match part.to_lowercase().as_str() {
            // 修饰键
            "ctrl" | "control" => modifiers.push(enigo::Key::Control),
            "shift" => modifiers.push(enigo::Key::Shift),
            "alt" => modifiers.push(enigo::Key::Alt),
            "meta" | "cmd" | "win" => modifiers.push(enigo::Key::Meta),

            // 主操作 - 鼠标按钮
            "mouseleft" => set_primary(
                &mut primary_action,
                PrimaryAction::MouseClick {
                    button: enigo::Button::Left,
                },
            )?,
            "mouseright" => set_primary(
                &mut primary_action,
                PrimaryAction::MouseClick {
                    button: enigo::Button::Right,
                },
            )?,
            "mousemiddle" => set_primary(
                &mut primary_action,
                PrimaryAction::MouseClick {
                    button: enigo::Button::Middle,
                },
            )?,
            "mousex1" => set_primary(
                &mut primary_action,
                PrimaryAction::MouseClick {
                    button: enigo::Button::Forward,
                },
            )?,
            "mousex2" => set_primary(
                &mut primary_action,
                PrimaryAction::MouseClick {
                    button: enigo::Button::Back,
                },
            )?,

            // 主操作 - 鼠标滚轮
            "mousewheelup" => set_primary(
                &mut primary_action,
                PrimaryAction::MouseWheel { amount: -1 },
            )?,
            "mousewheeldown" => {
                set_primary(&mut primary_action, PrimaryAction::MouseWheel { amount: 1 })?
            }

            "virtualkeyboard" => {
                set_primary(
                    &mut primary_action,
                    PrimaryAction::VirtualKeyboard {
                        virtual_keyboard: Some(true),
                    },
                )?;
            }

            // 主操作 - 其他键盘按键
            key_str => {
                let key = match key_str {
                    "space" => enigo::Key::Space,
                    "enter" => enigo::Key::Unicode('\r'),
                    "backspace" => enigo::Key::Backspace,
                    "tab" => enigo::Key::Tab,
                    "delete" => enigo::Key::Delete,
                    "pageup" => enigo::Key::PageUp,
                    "pagedown" => enigo::Key::PageDown,
                    "home" => enigo::Key::Home,
                    "end" => enigo::Key::End,
                    "capslock" => enigo::Key::CapsLock,
                    "printscreen" => enigo::Key::PrintScr,
                    "scrolllock" => enigo::Key::Scroll,
                    "pause" => enigo::Key::Pause,
                    s if s.starts_with("f") && s.len() > 1 && s[1..].chars().all(|c| c.is_ascii_digit()) => {
                        // 功能键 F1-F24
                        let num_str = &s[1..];
                        if let Ok(num) = num_str.parse::<u8>() {
                            match num {
                                1..=24 => get_function_key(num),
                                _ => return Err(ParseError::UnknownKey(key_str.to_string())),
                            }
                        } else {
                            return Err(ParseError::UnknownKey(key_str.to_string()));
                        }
                    }
                    // 匹配单个字符的按键
                    s if s.len() == 1 => enigo::Key::Unicode(s.chars().next().unwrap()),
                    _ => return Err(ParseError::UnknownKey(key_str.to_string())),
                };
                set_primary(&mut primary_action, PrimaryAction::KeyPress { key })?;
            }
        }
    }

    // 如果没有主操作但有修饰键，保持修饰键不变，使用空操作作为主操作
    if let Some(primary) = primary_action {
        Ok(Action { modifiers, primary })
    } else if !modifiers.is_empty() {
        // 保持修饰键不变，使用空操作作为主操作
        Ok(Action {
            modifiers,
            primary: PrimaryAction::None { none: None },
        })
    } else {
        Err(ParseError::NoPrimaryAction)
    }
}

// --- 工作线程和初始化 (ﾉ´▽｀)ﾉ♪ ---

/// Enigo 工作线程，接收命令并执行实际的键盘/鼠标操作。
/// 所有 Enigo 的操作都在这个线程中完成，以避免与主线程的阻塞和冲突。
fn enigo_worker(rx: Receiver<EnigoCommand>) {
    let enigo = Enigo::new(&enigo::Settings::default()).unwrap();
    *GLOBAL_ENIGO.write().unwrap() = Some(enigo);

    while let Ok(command) = rx.recv() {
        if let Some(enigo_instance) = GLOBAL_ENIGO.write().unwrap().as_mut() {
            match command {
                EnigoCommand::Execute(action) => {
                    action.execute(enigo_instance);
                }
                EnigoCommand::ExecutePress(action) => {
                    action.execute_press(enigo_instance);
                }
                EnigoCommand::ExecuteRelease(action) => {
                    action.execute_release(enigo_instance);
                }
            }
        } else {
            log::error!("Enigo 实例未初始化，无法执行操作");
        }
    }
}

/// 初始化函数，在程序启动时调用。
/// 主要用于加载映射配置，确保程序可以正常运行。
pub fn initialize() {
    log::debug!("初始化映射模块");
    // 确保全局映射缓存已加载
    load_mappings();
    // 强制初始化 ENIGO_SENDER，确保 enigo_worker 线程提前启动
    init_enigo_sender();
}

/// 强制初始化 ENIGO_SENDER，确保 enigo_worker 线程提前启动
pub fn init_enigo_sender() {
    log::debug!("初始化 ENIGO_SENDER");
    // 通过访问 ENIGO_SENDER 来强制初始化它
    let _ = &*ENIGO_SENDER;
    log::debug!("ENIGO_SENDER 初始化完成");
}

fn handle_trigger_data(controller_datas: &mut ControllerDatas, mapping: &Mapping) {
    let trigger_some = match mapping.composed_button.as_str() {
        "LT" => Some(controller_datas.left_trigger),
        "RT" => Some(controller_datas.right_trigger),
        _ => None,
    };

    if let Some(mut trigger) = trigger_some {
        trigger.check_triggered(Some(mapping.trigger_theshold));
        // 根据映射的按键设置对应的按钮状态
        match mapping.composed_button.as_str() {
            "LT" => controller_datas.set_button(ControllerButtons::LT, trigger.is_triggered()),
            "RT" => controller_datas.set_button(ControllerButtons::RT, trigger.is_triggered()),
            _ => {}
        }
    }
}


// 统计分类mapping，预计在映射更新时被统一调用
pub fn map_classification(mapping: &Vec<Mapping>) {
    let max = MappingTag::Tail as u8;

    // for mapping in mapping.iter() {
    //     if let Some(tag) = mapping.tag {
    //         tag_counts[tag as usize] += 1;
    //     }
    // }

    // log::debug!("Mapping classification: {:?}", tag_counts);
}

pub fn deal_normal_mapping(controller_datas: &mut ControllerDatas, mapping: &Mapping) {
    // 处理普通映射的逻辑
}

/// 核心映射函数，将手柄输入映射到相应的操作。
/// 遍历所有映射配置，检查手柄状态，并触发相应的操作。
pub fn map(controller_datas: &mut ControllerDatas, use_sub_preset: bool) {
    let mut mappings = if use_sub_preset {
        SUB_MAPPING_CACHE.read().unwrap().clone()
    } else {
        GLOBAL_MAPPING_CACHE.read().unwrap().clone()
    };

    // mappings.retain_mut(|mapping| {
    //     let tag = mapping.mapping_tag.as_ref().unwrap_or(&MappingTag::Normal);
    //     match tag {
    //         MappingTag::Group => {
    //             // _map(controller_datas, mappings.clone(), use_sub_preset);
    //         }
    //         MappingTag::Normal => {
    //             // 继续处理正常映射
    //             deal_normal_mapping(controller_datas, mapping);
    //         }
    //         _ => {
    //         }
    //     }
    //     false
    // });

    let layout_map = get_current_controller_layout_map();
    let mut trigger_states = DYNAMIC_TRIGGER_STATES.write().unwrap();
    let mut button_check_states = BUTTON_CHECK_STATES.write().unwrap();


    for mapping in mappings.iter() {
        let composed_button = mapping.get_composed_button();

        // 检查是否为摇杆旋转映射
        let rotation_match = match composed_button {
            "LeftStickCW" => {
                Some(controller_datas.left_stick_rotation == JoystickRotation::Clockwise)
            }
            "LeftStickCCW" => {
                Some(controller_datas.left_stick_rotation == JoystickRotation::CounterClockwise)
            }
            "RightStickCW" => {
                Some(controller_datas.right_stick_rotation == JoystickRotation::Clockwise)
            }
            "RightStickCCW" => {
                Some(controller_datas.right_stick_rotation == JoystickRotation::CounterClockwise)
            }
            _ => None,
        };

        if let Some(is_rotating) = rotation_match {
            // --- 处理摇杆旋转映射 (虚拟按键) ---
            let trigger_state = trigger_states
                .entry(mapping.get_id())
                .or_insert_with(|| mapping.trigger_state.clone());

            if trigger_state.should_trigger(is_rotating) {
                if trigger_state.continually_trigger {
                    // 连续触发模式：使用原有的 Execute 命令
                    ENIGO_SENDER
                        .send(EnigoCommand::Execute(mapping.action.clone()))
                        .unwrap();
                } else {
                    // 非连续触发模式：根据按键状态发送按下或释放命令
                    if trigger_state.is_key_pressed() {
                        ENIGO_SENDER
                            .send(EnigoCommand::ExecutePress(mapping.action.clone()))
                            .unwrap();
                    } else {
                        ENIGO_SENDER
                            .send(EnigoCommand::ExecuteRelease(mapping.action.clone()))
                            .unwrap();
                    }
                }
            } else if !is_rotating && trigger_state.continually_trigger {
                // 连续触发模式下，如果摇杆停止旋转，重置触发状态
                trigger_state.reset();
            }
        } else if let Some(button) = layout_map.get(composed_button) {
            handle_trigger_data(controller_datas, mapping);

            // --- 按键检测逻辑 ---
            let button_is_pressed = controller_datas.get_button(*button);
            let check_state = button_check_states.entry(mapping.get_id()).or_default();

            // 先进行按键检测，根据检测结果决定是否继续执行映射
            let should_trigger_mapping = check_button_press(
                button_is_pressed,
                mapping.check_mode,
                mapping.check_mode_param,
                check_state,
            );

            // log::error!("Mapping ID: {}, Button: {}, Pressed: {}, Mode: {:#?}, Should Trigger: {}", mapping.get_id(), composed_button, button_is_pressed, mapping.check_mode, should_trigger_mapping);

            // --- 处理原始按键映射 ---
            let trigger_state = trigger_states
                .entry(mapping.get_id())
                .or_insert_with(|| mapping.trigger_state.clone());

            // 在非连续触发模式下，需要处理按键释放的情况，确保按键能正确释放
            if !trigger_state.continually_trigger
                && !button_is_pressed
                && trigger_state.is_key_pressed()
            {
                // 非连续触发模式下，如果按键被释放但状态还是按下，需要触发释放操作
                trigger_state.set_key_pressed(false);
                ENIGO_SENDER
                    .send(EnigoCommand::ExecuteRelease(mapping.action.clone()))
                    .unwrap();
            }

            // 处理按键释放时的重置逻辑
            if !button_is_pressed {
                if trigger_state.continually_trigger {
                    // 连续触发模式下，如果按键被释放，重置触发状态
                    trigger_state.reset();
                }
                // 长按模式下，如果按键被释放，重置触发状态
                if mapping.check_mode == CheckMode::Long {
                    trigger_state.reset();
                }
            }

            // 只有当按键检测通过时，才执行原有的触发逻辑
            if should_trigger_mapping && trigger_state.should_trigger(button_is_pressed) {
                if trigger_state.continually_trigger {
                    // 连续触发模式：使用原有的 Execute 命令
                    ENIGO_SENDER
                        .send(EnigoCommand::Execute(mapping.action.clone()))
                        .unwrap();
                } else {
                    // 非连续触发模式：根据按键状态发送按下或释放命令
                    if trigger_state.is_key_pressed() {
                        ENIGO_SENDER
                            .send(EnigoCommand::ExecutePress(mapping.action.clone()))
                            .unwrap();
                    } else {
                        ENIGO_SENDER
                            .send(EnigoCommand::ExecuteRelease(mapping.action.clone()))
                            .unwrap();
                    }
                }
            }
        }
    }
}

pub struct MouseMovementState {
    move_x_remainder: f32,
    move_y_remainder: f32,
}

impl Default for MouseMovementState {
    fn default() -> Self {
        Self {
            move_x_remainder: 0.0,
            move_y_remainder: 0.0,
        }
    }
}

static MOUSE_MOVEMENT_STATE: Lazy<RwLock<MouseMovementState>> =
    Lazy::new(|| RwLock::new(MouseMovementState::default()));

static GLOBAL_ENIGO: Lazy<RwLock<Option<Enigo>>> = Lazy::new(|| RwLock::new(None));

pub fn handle_mouse_movement(controller_datas: &ControllerDatas) {
    let (move_speed, stick_as_mouse_simulation) = {
        let preset = preset::get_current_preset();
        if !preset.items.use_stick_as_mouse {
            return;
        }
        (
            preset.items.move_speed,
            preset.items.stick_as_mouse_simulation,
        )
    };

    let (stick_x, stick_y) = match stick_as_mouse_simulation {
        Some(which_stick) => match which_stick.as_str() {
            "left" => (controller_datas.left_stick.x, controller_datas.left_stick.y),
            "right" => (
                controller_datas.right_stick.x,
                controller_datas.right_stick.y,
            ),
            "touchpad" => (0.0, 0.0), // TODO: 触摸板模拟
            _ => {
                return;
            }
        },
        None => {
            // log::warn!("未指定用于鼠标模拟的摇杆");
            return;
        }
    };

    // --- 精度累积计算 ---
    let (move_x, move_y) = {
        let mut state = MOUSE_MOVEMENT_STATE.write().unwrap();

        // 直接在状态上进行累积计算
        state.move_x_remainder += stick_x * (move_speed as f32);
        state.move_y_remainder += -stick_y * (move_speed as f32); // Y轴反转，以匹配屏幕坐标

        // 取出整数部分进行移动
        let move_x = state.move_x_remainder.trunc();
        let move_y = state.move_y_remainder.trunc();

        // 从累积值中减去已移动的部分
        state.move_x_remainder -= move_x;
        state.move_y_remainder -= move_y;

        (move_x, move_y)
    };

    if move_x != 0.0 || move_y != 0.0 {
        let mut enigo = GLOBAL_ENIGO.write().unwrap();
        if let Some(enigo_instance) = enigo.as_mut() {
            enigo_instance
                .move_mouse(move_x as i32, move_y as i32, enigo::Coordinate::Rel)
                .expect("error");
        } else {
            log::error!("enigo error")
        }
    }
}
