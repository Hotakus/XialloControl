#![allow(dead_code)]

// --- 依赖项和常量 ฅ^•ﻌ•^ฅ ---
use crate::controller::controller::{CURRENT_DEVICE, ControllerType};
use crate::controller::datas::{ControllerButtons, ControllerDatas};
use crate::xeno_utils;
use enigo::{Enigo, Keyboard, Mouse};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

// --- 数据结构定义 (•̀ω•́)✧ ---

/// 触发状态，用于控制按键的重复触发和加速。
/// 当按键被持续按下时，它会以一定的间隔重复触发，并且间隔会逐渐减小（加速）。
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TriggerState {
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
}

impl TriggerState {
    /// 默认的 `Instant::now()` 函数，用于反序列化时初始化。
    fn default_instant() -> Instant {
        Instant::now()
    }

    /// 创建一个带有默认参数的 `TriggerState`。
    pub fn default() -> Self {
        Self::new(300, 100, 0.8)
    }

    /// 使用指定的参数创建一个新的 `TriggerState`。
    pub fn new(initial_interval: u64, min_interval: u64, acceleration: f64) -> Self {
        Self {
            interval: initial_interval,
            initial_interval,
            min_interval,
            acceleration,
            last_trigger: Instant::now(),
        }
    }

    /// 检查是否应该触发。如果自上次触发以来经过的时间超过了当前间隔，则返回 `true`。
    /// 触发后，会自动更新下一次的触发间隔（加速）。
    pub fn should_trigger(&mut self) -> bool {
        if self.last_trigger.elapsed().as_millis() as u64 >= self.interval {
            // 触发后加速，更新间隔时间
            self.interval =
                ((self.interval as f64) * self.acceleration).max(self.min_interval as f64) as u64;
            self.last_trigger = Instant::now();
            true
        } else {
            false
        }
    }

    /// 重置触发状态到初始间隔。
    pub fn reset(&mut self) {
        self.interval = self.initial_interval;
    }
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

    #[serde(flatten)]
    /// 解析后的实际操作指令。
    action: Action,

    /// 触发选项，用于控制按键的重复触发行为。
    #[serde(flatten)]
    trigger_state: TriggerState,
}

impl Mapping {
    /// 创建一个带有默认 `Action` 和 `TriggerState` 的新 `Mapping`。
    pub fn new(id: u64, composed_button: String, composed_shortcut_key: String) -> Self {
        Self {
            id,
            composed_button,
            composed_shortcut_key,
            action: Action::default(),
            trigger_state: TriggerState::default(),
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

impl Action {
    /// 创建一个默认的 `Action`，其主要操作为按下空格键。
    pub fn default() -> Action {
        Action {
            modifiers: vec![],
            primary: PrimaryAction::KeyPress {
                key: enigo::Key::Space,
            },
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

/// Enigo 命令类型，用于工作线程间通信。
#[derive(Debug)]
pub enum EnigoCommand {
    /// 执行一个 `Action`。
    Execute(Action),
}

// --- 全局静态变量 (づ￣ 3￣)づ ---

/// 全局映射配置缓存，使用 `RwLock` 确保线程安全。
pub static GLOBAL_MAPPING_CACHE: Lazy<RwLock<Vec<Mapping>>> = Lazy::new(|| {
    let mappings = vec![];
    RwLock::new(mappings)
});

/// TOML 配置文件名。
const DEFAULT_MAPPINGS_FILE: &str = "mappings.toml";
pub static MAPPING_FILE_PATH: Lazy<RwLock<PathBuf>> =
    Lazy::new(|| RwLock::from(PathBuf::from(DEFAULT_MAPPINGS_FILE)));

/// 全局手柄按键布局映射，例如将 "Y" 映射到 `ControllerButtons::North`。
/// 存储不同类型手柄的布局。
pub static CONTROLLER_LAYOUT_MAP: Lazy<RwLock<HashMap<ControllerType, Arc<HashMap<&'static str, ControllerButtons>>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// 动态触发状态，存储每个映射的触发状态。
pub static DYNAMIC_TRIGGER_STATES: Lazy<RwLock<HashMap<u64, TriggerState>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Enigo 工作线程的发送器，用于向其发送执行命令。
pub static ENIGO_SENDER: Lazy<Sender<EnigoCommand>> = Lazy::new(|| {
    let (tx, rx): (Sender<EnigoCommand>, Receiver<EnigoCommand>) = channel();
    // 启动工作线程来处理 Enigo 命令
    thread::spawn(move || enigo_worker(rx));
    tx
});

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

    if !mappings_path.exists() {
        log::warn!("映射配置文件不存在，将创建空文件: {mappings_path:#?}");
        // 创建空映射文件
        let mapping_file = MappingFile { mappings: vec![] };
        if let Err(e) = xeno_utils::write_toml_file(&mappings_path, &mapping_file) {
            log::error!("创建空映射文件失败: {e}");
        }
        return vec![];
    }

    match xeno_utils::read_toml_file::<MappingFile>(&mappings_path) {
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
        Ok(_) => log::info!("映射配置已保存到: {mappings_path:#?}", ),
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

/// Tauri 命令：更新一个已存在的映射配置。
#[tauri::command]
pub fn update_mapping(id: u64, composed_button: String, composed_shortcut_key: String, trigger_state: TriggerState) -> bool {
    match parse_composed_key_to_action(&composed_shortcut_key) {
        Ok(action) => {
            let mut cache = GLOBAL_MAPPING_CACHE.write().unwrap();
            if let Some(mapping) = cache.iter_mut().find(|m| m.id == id) {
                mapping.composed_button = composed_button;
                mapping.composed_shortcut_key = composed_shortcut_key;
                mapping.action = action;
                mapping.trigger_state = trigger_state.clone();

                let mut live_triggers = DYNAMIC_TRIGGER_STATES.write().unwrap();
                live_triggers.insert(id, trigger_state);
                drop(live_triggers);

                drop(cache);
                save_mappings();
                return true;
            }
            log::error!("更新失败，未找到 id {id} 的映射");
            false
        }
        Err(e) => {
            log::error!("解析快捷键失败 '{composed_shortcut_key}': {e:?}");
            false
        }
    }
}

/// Tauri 命令：添加一个新的映射配置。
#[tauri::command]
pub fn add_mapping(composed_button: String, composed_shortcut_key: String, trigger_state: TriggerState) -> bool {
    log::debug!(
        "请求添加映射配置: '{composed_button}', '{composed_shortcut_key}'",
    );
    // 调用解析器来生成 Action
    match parse_composed_key_to_action(&composed_shortcut_key) {
        Ok(action) => {
            let mut cache = GLOBAL_MAPPING_CACHE.write().unwrap();
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            // 创建包含 Action 的新 Mapping
            let mapping = Mapping {
                id,
                composed_button,
                composed_shortcut_key,
                action, // <-- 存储解析结果
                trigger_state, // <-- 使用前端传来的触发状态
            };

            cache.push(mapping);
            drop(cache);
            save_mappings();
            true
        }
        Err(e) => {
            log::error!("解析快捷键失败 '{composed_shortcut_key}': {e:?}");
            false
        }
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
        switch_map.insert("A", ControllerButtons::East);  // Switch 的 A 对应 Xbox 的 B
        switch_map.insert("R", ControllerButtons::RB);    // Switch 的 R 对应 Xbox 的 RB
        switch_map.insert("L", ControllerButtons::LB);    // Switch 的 L 对应 Xbox 的 LB
        switch_map.insert("LeftStick", ControllerButtons::LStick);
        switch_map.insert("RightStick", ControllerButtons::RStick);
        switch_map.insert("Minus", ControllerButtons::Back);   // Switch 的 Minus 对应 Xbox 的 Back
        switch_map.insert("Plus", ControllerButtons::Start);   // Switch 的 Plus 对应 Xbox 的 Start
        switch_map.insert("Home", ControllerButtons::Guide);   // Switch 的 Home 对应 Xbox 的 Guide
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
        map.insert(ControllerType::PlayStation, Arc::new(create_playstation_layout_map()));
        map.insert(ControllerType::Switch, Arc::new(create_switch_layout_map())); // 添加 Switch 布局
        map.insert(ControllerType::Other, Arc::new(create_other_layout_map()));
    }
}

/// 获取当前连接手柄的按键布局映射的只读引用。
fn get_current_controller_layout_map() -> Arc<HashMap<&'static str, ControllerButtons>> {
    init_controller_layout_maps();
    let controller_type = CURRENT_DEVICE.read().unwrap().controller_type;
    let map_guard = CONTROLLER_LAYOUT_MAP.read().unwrap();
    map_guard.get(&controller_type)
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

            // 主操作 - 其他键盘按键
            key_str => {
                let key = match key_str {
                    "space" => enigo::Key::Space,
                    "enter" => enigo::Key::Unicode('\r'),
                    // 匹配单个字符的按键
                    s if s.len() == 1 => enigo::Key::Unicode(s.chars().next().unwrap()),
                    _ => return Err(ParseError::UnknownKey(key_str.to_string())),
                };
                set_primary(&mut primary_action, PrimaryAction::KeyPress { key })?;
            }
        }
    }

    if let Some(primary) = primary_action {
        Ok(Action { modifiers, primary })
    } else {
        Err(ParseError::NoPrimaryAction)
    }
}

// --- 工作线程和初始化 (ﾉ´▽｀)ﾉ♪ ---

/// Enigo 工作线程，接收命令并执行实际的键盘/鼠标操作。
/// 所有 Enigo 的操作都在这个线程中完成，以避免与主线程的阻塞和冲突。
fn enigo_worker(rx: Receiver<EnigoCommand>) {
    // 创建 Enigo 实例
    let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();

    while let Ok(cmd) = rx.recv() {
        match cmd {
            EnigoCommand::Execute(action) => {
                // 1. 按下所有修饰键
                for modifier in &action.modifiers {
                    enigo
                        .key(*modifier, enigo::Direction::Press)
                        .expect("Failed to press modifier key");
                }

                // 2. 执行主操作
                match action.primary {
                    PrimaryAction::KeyPress { key } => {
                        enigo
                            .key(key, enigo::Direction::Click)
                            .expect("Failed to press key"); // 按下并释放
                    }
                    PrimaryAction::MouseClick { button } => {
                        enigo
                            .button(button, enigo::Direction::Click)
                            .expect("Failed to clicked mouse button");
                    }
                    PrimaryAction::MouseWheel { amount } => {
                        enigo
                            .scroll(amount, enigo::Axis::Vertical)
                            .expect("Failed to scroll mouse weight");
                    }
                }

                // 3. 释放所有修饰键 (以相反顺序)
                for modifier in action.modifiers.iter().rev() {
                    enigo
                        .key(*modifier, enigo::Direction::Release)
                        .expect("Failed to release modifier key");
                }
            }
        }
    }
}

/// 初始化函数，在程序启动时调用。
/// 主要用于加载映射配置，确保程序可以正常运行。
pub fn initialize() {
    log::debug!("初始化映射模块");
    // 确保全局映射缓存已加载
    load_mappings();
}

/// 核心映射函数，将手柄输入映射到相应的操作。
/// 遍历所有映射配置，检查手柄状态，并触发相应的操作。
pub fn map(controller_datas: &ControllerDatas) {
    // 获取可写的映射配置和触发状态，以及只读的布局映射
    let mut mappings = GLOBAL_MAPPING_CACHE.write().unwrap();

    let layout_map = get_current_controller_layout_map();

    let mut trigger_states = DYNAMIC_TRIGGER_STATES.write().unwrap();

    // 遍历所有映射
    for mapping in mappings.iter_mut() {
        // 根据手柄按钮字符串获取对应的枚举值
        if let Some(button) = layout_map.get(mapping.get_composed_button()) {
            // 检查该手柄按钮是否被按下
            if controller_datas.get_button(*button) {
                // 获取或插入该映射的触发状态
                let trigger_state = trigger_states
                    .entry(mapping.get_id())
                    .or_insert_with(|| mapping.trigger_state.clone());

                // 检查是否可以触发操作
                if trigger_state.should_trigger() {
                    // 只需发送解析好的 Action 即可！
                    ENIGO_SENDER
                        .send(EnigoCommand::Execute(mapping.action.clone()))
                        .unwrap();
                }
            } else if let Some(state) = trigger_states.get_mut(&mapping.get_id()) {
                // 如果按钮未被按下，重置触发状态
                state.reset();
            }
        }
    }
}
