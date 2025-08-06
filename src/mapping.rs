use crate::controller::controller::{ControllerType, DeviceInfo};
use crate::controller::datas::{ControllerButtons, ControllerDatas};
use crate::xeno_utils;
use enigo::{Enigo, Keyboard, Mouse};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MappingType {
    Keyboard,
    MouseButton,
    MouseWheel
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Mapping {
    id: u64,
    composed_button: String,
    composed_shortcut_key: String,
    mapping_type: MappingType,
}

impl Mapping {
    pub fn new(
        id: u64,
        composed_button: String,
        composed_shortcut_key: String,
        mapping_type: MappingType,
    ) -> Self {
        Self {
            id,
            composed_button,
            composed_shortcut_key,
            mapping_type,
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_controller_button(&self) -> &str {
        &self.composed_button
    }

    pub fn get_composed_key(&self) -> &str {
        &self.composed_shortcut_key
    }

    pub fn get_mapping_type(&self) -> MappingType {
        self.mapping_type.clone()
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

pub static GLOBAL_ENIGO: Lazy<RwLock<Enigo>> =
    Lazy::new(|| RwLock::from(Enigo::new(&enigo::Settings::default()).unwrap()));

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

use std::sync::Mutex;
use std::time::{Duration, Instant};

// 🧠 长按状态缓存
#[derive(Clone)]
struct RepeatState {
    press_start: Instant,
    last_fire: Instant,
    interval: Duration,
}

static REPEAT_STATES: Lazy<Mutex<HashMap<ControllerButtons, RepeatState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// 🔧 调整参数：初始延迟、起始间隔、最小间隔、递减倍率
const INITIAL_DELAY: Duration = Duration::from_millis(400);
const START_INTERVAL: Duration = Duration::from_millis(300);
const MIN_INTERVAL: Duration = Duration::from_millis(50);
const INTERVAL_DECAY: f32 = 0.85;

pub fn map(device: DeviceInfo, controller_datas: ControllerDatas) {
    let mut enigo = GLOBAL_ENIGO.write().unwrap();
    let mappings = get_mappings_internal();
    let mut layout_map: HashMap<&'static str, ControllerButtons>;

    let mut repeat_states = REPEAT_STATES.lock().unwrap();
    let now = Instant::now();


    for mapping in mappings {
        match device.controller_type {
            ControllerType::Xbox => {
                layout_map = get_xbox_layout_map().clone();
            }
            _ => {
                layout_map = get_xbox_layout_map().clone();
            }
        }

        if let Some(button) = layout_map.get(mapping.get_controller_button()) {


            match mapping.get_mapping_type() {
                MappingType::Keyboard => {
                    // TODO: Keyboard
                    let is_pressed = controller_datas.get_button(*button);
                    if is_pressed {
                        let res = parse_composed_key(mapping.get_composed_key());
                        press_composed_keys(&mut enigo, &res);
                    }
                }
                MappingType::MouseButton => {
                    // TODO: MouseButton
                }
                MappingType::MouseWheel => {
                    // TODO: MouseWheel
                }
            }
        }
    }
}


// if is_pressed {
// if let Some(state) = repeat_states.get_mut(button) {
// // 是否超过初始延迟？
// if now.duration_since(state.press_start) < INITIAL_DELAY {
// continue;
// }
//
// // 是否到了触发时间？
// if now.duration_since(state.last_fire) >= state.interval {
// let composed = parse_composed_key(mapping.get_composed_key());
// press_composed_keys(&mut enigo, &composed);
//
// // 更新状态
// state.last_fire = now;
// state.interval = (state.interval.mul_f32(INTERVAL_DECAY)).max(MIN_INTERVAL);
// }
// } else {
// // 初次按下立即触发
// let composed = parse_composed_key(mapping.get_composed_key());
// press_composed_keys(&mut enigo, &composed);
//
// repeat_states.insert(*button, RepeatState {
// press_start: now,
// last_fire: now,
// interval: START_INTERVAL,
// });
// }
// } else {
// // 如果释放就清除
// repeat_states.remove(button);
// }
