#![allow(dead_code)]

use crate::controller::controller::{DeviceInfo};
use crate::controller::datas::{ControllerButtons, ControllerDatas};
use crate::xeno_utils;
use enigo::{Enigo, Keyboard, Mouse};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{RwLock, RwLockReadGuard};
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TriggerState {
    interval: u64,
    initial_interval: u64,
    min_interval: u64,
    acceleration: f64,

    #[serde(skip)] // 跳过序列化和反序列化
    #[serde(default = "TriggerState::default_instant")] // 反序列化时自动调用
    last_trigger: Instant,
}

impl TriggerState {
    fn default_instant() -> Instant {
        Instant::now()
    }
    pub fn default() -> Self {
        Self::new(300, 100, 0.8)
    }
    pub fn new(initial_interval: u64, min_interval: u64, acceleration: f64) -> Self {
        Self {
            interval: initial_interval,
            initial_interval,
            min_interval,
            acceleration,
            last_trigger: Instant::now(),
        }
    }

    pub fn should_trigger(&mut self) -> bool {
        if self.last_trigger.elapsed().as_millis() as u64 >= self.interval {
            // 触发后加速
            self.interval =
                ((self.interval as f64) * self.acceleration).max(self.min_interval as f64) as u64;
            self.last_trigger = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.interval = self.initial_interval;
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Mapping {
    id: u64,
    composed_button: String,
    composed_shortcut_key: String,

    #[serde(flatten)]
    action: Action,

    // 触发选项
    #[serde(flatten)]
    trigger_state: TriggerState,
}

impl Mapping {
    pub fn new(
        id: u64,
        composed_button: String,
        composed_shortcut_key: String,
    ) -> Self {
        Self {
            id,
            composed_button,
            composed_shortcut_key,
            action: Action::default(),
            trigger_state: TriggerState::default(),
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_composed_button(&self) -> &str {
        &self.composed_button
    }

    pub fn get_composed_key(&self) -> &str {
        &self.composed_shortcut_key
    }
}

// 包装结构体用于文件序列化
#[derive(Serialize, Deserialize)]
struct MappingFile {
    mappings: Vec<Mapping>,
}

// pub static GLOBAL_MAPPING_CACHE: Lazy<RwLock<Vec<Mapping>>> =
//     Lazy::new(|| RwLock::new(load_mappings_internal()));

pub static GLOBAL_MAPPING_CACHE: Lazy<RwLock<Vec<Mapping>>> = Lazy::new(|| {
    let mappings = vec![];
    RwLock::new(mappings)
});

const MAPPINGS_FILE: &str = "mappings.toml";

/// 内部加载映射实现
fn load_mappings_internal() -> Vec<Mapping> {
    let mappings_path = xeno_utils::get_config_path(MAPPINGS_FILE);

    if !mappings_path.exists() {
        log::warn!("映射配置文件不存在，将创建空文件");
        // 创建空映射文件
        let mapping_file = MappingFile { mappings: vec![] };
        if let Err(e) = xeno_utils::write_toml_file(&mappings_path, &mapping_file) {
            log::error!("创建空映射文件失败: {}", e);
        }
        return vec![];
    }

    match xeno_utils::read_toml_file::<MappingFile>(&mappings_path) {
        Ok(mapping_file) => {
            log::info!("成功加载 {} 条映射配置", mapping_file.mappings.len());
            mapping_file.mappings
        }
        Err(e) => {
            log::error!("加载映射配置失败: {}", e);
            vec![]
        }
    }
}

/// 加载应用到全局映射缓存
pub fn load_mappings() {
    let mut cache = GLOBAL_MAPPING_CACHE.write().unwrap();
    *cache = load_mappings_internal();
    log::info!("映射缓存已加载 {:#?}", cache);
}

/// 保存全局映射缓存到文件
pub fn save_mappings() {
    // 确保配置目录存在
    xeno_utils::ensure_config_dir();

    let mappings = get_mappings_internal();
    let mappings_path = xeno_utils::get_config_path(MAPPINGS_FILE);

    let mapping_file = MappingFile {
        mappings: mappings.clone(),
    };

    match xeno_utils::write_toml_file(&mappings_path, &mapping_file) {
        Ok(_) => log::info!("映射配置已保存到: {:?}", mappings_path),
        Err(e) => log::error!("保存映射配置失败: {:#?}", e),
    }
}

/// 获取当前映射（线程安全）
pub fn get_mappings_internal() -> Vec<Mapping> {
    GLOBAL_MAPPING_CACHE.read().unwrap().clone()
}

#[tauri::command]
pub fn set_mapping(mapping: Vec<Mapping>) {
    log::debug!("更新映射配置: {:#?}", mapping);
    {
        let mut cache = GLOBAL_MAPPING_CACHE.write().unwrap();
        *cache = mapping;
    }
    save_mappings(); // 更新后立即保存
    log::debug!("映射缓存已更新并保存");
}

#[tauri::command]
pub fn update_mapping(id: u64, composed_button: String, composed_shortcut_key: String) -> bool {
    match parse_composed_key_to_action(&composed_shortcut_key) {
        Ok(action) => {
            let mut cache = GLOBAL_MAPPING_CACHE.write().unwrap();
            if let Some(mapping) = cache.iter_mut().find(|m| m.id == id) {
                mapping.composed_button = composed_button;
                mapping.composed_shortcut_key = composed_shortcut_key;
                mapping.action = action; // <-- 更新解析结果

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

#[tauri::command]
pub fn add_mapping(composed_button: String, composed_shortcut_key: String) -> bool {
    log::debug!("请求添加映射配置: '{}', '{}'", composed_button, composed_shortcut_key);
    // 调用解析器
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
                trigger_state: TriggerState::default(),
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

#[tauri::command]
pub fn delete_mapping(id: u64) -> bool {
    log::debug!("请求删除 id {} 的映射配置", id);
    let mut cache = GLOBAL_MAPPING_CACHE.write().unwrap();
    let initial_len = cache.len();

    // 使用 retain 方法高效地移除指定 id 的项
    cache.retain(|m| m.get_id() != id);

    // 检查是否有元素被真的删除了
    let deleted = cache.len() < initial_len;

    if deleted {
        // 因为状态已改变，所以需要保存
        // 在调用 save_mappings 之前释放锁，避免死锁
        drop(cache);
        save_mappings();
        log::info!("已成功删除 id {} 的映射", id);
    } else {
        log::warn!("尝试删除一个不存在的映射 id: {}", id);
    }

    // 返回操作是否成功
    deleted
}

// 显式保存映射命令
#[tauri::command]
pub fn save_mapping_config() {
    log::debug!("前端请求保存映射配置");
    save_mappings();
}

// 获取当前映射配置
#[tauri::command]
pub fn get_mappings() -> Vec<Mapping> {
    get_mappings_internal()
}

pub fn initialize() {
    log::debug!("初始化映射模块");
    // 确保全局映射缓存已加载
    load_mappings();
}


fn press_composed_keys(enigo: &mut Enigo, keys: &[enigo::Key]) {
    // 按下组合键
    for key in keys {
        enigo.key(key.clone(), enigo::Direction::Press);
    }

    // 松开组合键（逆序）
    for key in keys.iter().rev() {
        enigo.key(key.clone(), enigo::Direction::Release);
    }
}


fn parse_composed_key(composed: &str) -> Vec<enigo::Key> {
    composed
        .split('+')
        .map(|k| k.trim())
        .filter_map(|k| match k.to_lowercase().as_str() {
            "ctrl" | "control" => Some(enigo::Key::Control),
            "shift" => Some(enigo::Key::Shift),
            "alt" => Some(enigo::Key::Alt),
            "meta" | "cmd" | "win" => Some(enigo::Key::Meta),
            "space" | " " => Some(enigo::Key::Unicode(' ')),
            // "MouseLeft" => Some(enigo::Button::),
            s if s.len() == 1 => {
                let c = s.chars().next().unwrap();
                Some(enigo::Key::Unicode(c))
            }
            _ => None,
        })
        .collect()
}

pub static XBOX_LAYOUT_MAP: Lazy<RwLock<HashMap<&'static str, ControllerButtons>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

fn init_if_needed() {
    let mut map = XBOX_LAYOUT_MAP.write().unwrap();
    if map.is_empty() {
        map.insert("Y", ControllerButtons::North);
        map.insert("X", ControllerButtons::West);
        map.insert("A", ControllerButtons::South);
        map.insert("B", ControllerButtons::East);
        map.insert("RB", ControllerButtons::RB);
        map.insert("LB", ControllerButtons::LB);
    }
}

fn get_xbox_layout_map() -> RwLockReadGuard<'static, HashMap<&'static str, ControllerButtons>> {
    init_if_needed();
    XBOX_LAYOUT_MAP.read().unwrap()
}

// --- 新增：定义主操作 ---
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case", untagged)]
pub enum PrimaryAction {
    KeyPress {
        #[serde(flatten)]
        key: enigo::Key
    },
    MouseClick { button: enigo::Button },
    MouseWheel { amount: i32 },
}

// --- 新增：定义完整的操作指令 ---
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Action {
    pub modifiers: Vec<enigo::Key>, // 需要按下的修饰键，如 [Key::Shift, Key::Control]

    #[serde(flatten)]
    pub primary: PrimaryAction, // 最终执行的核心动作
}

impl Action {
    pub fn default() -> Action {
        Action {
            modifiers: vec![],
            primary: PrimaryAction::KeyPress { key: enigo::Key::Space },
        }
    }
}

// 定义一个简单的错误类型
#[derive(Debug)]
pub enum ParseError {
    NoPrimaryAction,
    MultiplePrimaryActions,
    UnknownKey(String),
}

/// 解析按键组合字符串，生成结构化的 Action
fn parse_composed_key_to_action(composed: &str) -> Result<Action, ParseError> {
    let mut modifiers = Vec::new();
    let mut primary_action = None;

    for part in composed.split('+').map(|s| s.trim()) {
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
                    button: (enigo::Button::Right),
                },
            )?,
            "mousemiddle" => set_primary(
                &mut primary_action,
                PrimaryAction::MouseClick {
                    button: (enigo::Button::Middle),
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
                    button: (enigo::Button::Back),
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

            // 主操作 - 其他键盘按键 (简化处理)
            key_str => {
                // 这里可以用更复杂的逻辑来映射所有 enigo::Key
                // 例如 F1-F12, Space, Enter 等
                let key = match key_str {
                    "space" => enigo::Key::Space,
                    "enter" => enigo::Key::Return,
                    // ... 添加更多特殊键
                    s if s.len() == 1 => enigo::Key::Unicode(s.chars().next().unwrap()),
                    _ => return Err(ParseError::UnknownKey(key_str.to_string())),
                };
                set_primary(
                    &mut primary_action,
                    PrimaryAction::KeyPress {
                        key,
                    },
                )?;
            }
        }
    }

    if let Some(primary) = primary_action {
        Ok(Action { modifiers, primary })
    } else {
        Err(ParseError::NoPrimaryAction)
    }
}

// 辅助函数，确保只有一个主操作被设置
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

#[derive(Debug)]
pub enum EnigoCommand {
    Execute(Action),
}

pub static ENIGO_SENDER: Lazy<Sender<EnigoCommand>> = Lazy::new(|| {
    let (tx, rx): (Sender<EnigoCommand>, Receiver<EnigoCommand>) = channel();
    thread::spawn(move || enigo_worker(rx)); // 启动工作线程
    tx
});

fn enigo_worker(rx: Receiver<EnigoCommand>) {
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
                            .key(
                                key,
                                enigo::Direction::Click,
                            )
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
                    enigo.key(*modifier, enigo::Direction::Release)
                        .expect("Failed to release modifier key");
                }
            }
        }
    }
}

pub static KEYBOARD_TRIGGER_STATES_MAP: Lazy<RwLock<HashMap<ControllerButtons, TriggerState>>> =
    Lazy::new(|| RwLock::from(HashMap::new()));

pub static KEYBOARD_TRIGGER_STATES: Lazy<RwLock<TriggerState>> =
    Lazy::new(|| RwLock::from(TriggerState::new(300, 50, 0.9)));


pub static DYNAMIC_TRIGGER_STATES: Lazy<RwLock<HashMap<u64, TriggerState>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));


pub fn map(device: &DeviceInfo, controller_datas: &ControllerDatas) {
    let mut mappings = GLOBAL_MAPPING_CACHE.write().unwrap();
    let layout_map = get_xbox_layout_map(); // 假设
    let mut trigger_states = DYNAMIC_TRIGGER_STATES.write().unwrap();

    for mapping in mappings.iter_mut() {
        // 改为 iter_mut() 以便修改 trigger_state
        if let Some(button) = layout_map.get(mapping.get_composed_button()) {
            if controller_datas.get_button(*button) {
                let trigger_state = trigger_states
                    .entry(mapping.get_id())
                    .or_insert_with(|| mapping.trigger_state.clone());

                if trigger_state.should_trigger() {
                    // 只需发送解析好的 Action 即可！
                    ENIGO_SENDER
                        .send(EnigoCommand::Execute(mapping.action.clone()))
                        .unwrap();
                }
            } else if let Some(state) = trigger_states.get_mut(&mapping.get_id()) {
                state.reset();
            }
        }
    }
}
