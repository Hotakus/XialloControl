#![allow(dead_code)]

use crate::mapping::{Mapping, get_mappings};
use crate::setting::{get_setting, load_settings};
use crate::xeno_utils::ensure_dir;
use crate::{mapping, xeno_utils};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;

const PRESET_DIR: &str = "presets";
const DEFAULT_PRESET_NAME: &str = "default";
const DEFAULT_MAPPINGS_FILE: &str = "mappings.toml";

const DEFAULT_DEADZONE: u8 = 5; // 10%

pub static CURRENT_PRESET: Lazy<RwLock<Preset>> =
    Lazy::new(|| RwLock::new(Preset::new(DEFAULT_PRESET_NAME.to_string(), vec![])));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PresetItems {
    pub mappings_file_name: String,

    /// 右摇杆死区范围 (%)
    #[serde(default = "default_deadzone")]
    pub deadzone: u8,

    /// 左摇杆死区范围 (%)
    #[serde(default = "default_deadzone")]
    pub deadzone_left: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Preset {
    pub name: String,

    #[serde(flatten)]
    pub items: PresetItems,
}

impl Preset {
    pub fn new(name: String, mappings: Vec<Mapping>) -> Self {
        Self {
            name,
            items: PresetItems {
                mappings_file_name: DEFAULT_MAPPINGS_FILE.into(),
                deadzone: DEFAULT_DEADZONE,
                deadzone_left: DEFAULT_DEADZONE,
            },
        }
    }

    pub fn save(&self) -> bool {
        if let Some(save_dir) = ensure_dir(&PathBuf::from(PRESET_DIR).join(self.name.clone())) {
            let save_path = save_dir.join(self.name.clone() + ".toml");
            let save_data = PresetFile {
                preset: self.clone(),
            };

            return if let Err(e) = xeno_utils::write_toml_file(&save_path, &save_data) {
                log::error!("保存预设失败：{e}");
                false
            } else {
                true
            };
        }
        true
    }

    pub fn load(&mut self, name: &str) -> bool {
        if let Some(load_dir) = ensure_dir(&PathBuf::from(PRESET_DIR).join(name)) {
            let load_path = load_dir.join(name.to_string() + ".toml");
            if !load_path.exists() {
                log::warn!("预设文件不存在：{name}");
                self.name = name.to_string();
                return self.save();
            }

            let load_data = xeno_utils::read_toml_file::<PresetFile>(&load_path);
            match load_data {
                Ok(datas) => {
                    *self = datas.preset;
                    log::info!("加载预设：{load_path:#?}");
                    true
                }
                Err(e) => {
                    log::error!("加载预设失败：{e}");
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn set_mappings(&mut self, mappings: String) {
        self.items.mappings_file_name = mappings;
    }

    pub fn set_deadzone(&mut self, deadzone: u8) {
        self.items.deadzone = deadzone;
    }

    pub fn set_deadzone_left(&mut self, deadzone_left: u8) {
        self.items.deadzone_left = deadzone_left;
    }
}

#[derive(Serialize, Deserialize)]
struct PresetFile {
    pub preset: Preset,
}

fn default_deadzone() -> u8 {
    DEFAULT_DEADZONE
}

#[tauri::command]
pub fn preset_test() {
    let mut preset = CURRENT_PRESET.write().unwrap();
    let mut setting = load_settings();
    let mut mappings = get_mappings();

    // preset.set_deadzone(setting.deadzone);
    // preset.set_deadzone_left(setting.deadzone_left);
    // preset.set_mappings(mappings.clone());
    // preset.save();
}

#[tauri::command]
pub fn preset_test2() {
    let mut preset = Preset::new("test".to_string(), vec![]);
    preset.load("default");

    println!("{:#?}", preset.items.deadzone);
    println!("{:#?}", preset.items.deadzone_left);
    println!("{:#?}", preset.items.mappings_file_name);
}

// pub fn load_presets_list() -> Vec<Preset> {
//
// }

#[tauri::command]
pub fn load_preset(name: &str) -> Preset {
    let mut preset = CURRENT_PRESET.write().unwrap();
    preset.load(name);
    preset.clone()
}

pub fn get_current_preset() -> Preset {
    CURRENT_PRESET.read().unwrap().clone()
}

pub fn initialize() {
    log::info!("初始化预设");
    let mut preset = get_current_preset();
    let setting = get_setting();
    if preset.load(setting.previous_preset.as_str()) {
        // load mappings
        mapping::set_mapping_file_path(
            PathBuf::from(PRESET_DIR)
                .join(preset.name.clone())
                .join(preset.items.mappings_file_name.clone()),
        );
        println!(
            "mappings_file_path = {:#?}",
            mapping::get_mapping_file_path()
        );
    }
}

#[tauri::command]
pub fn update_deadzone(deadzone: u8, deadzone_left: u8) -> Result<(), String> {
    let mut preset = CURRENT_PRESET.write().unwrap();
    preset.set_deadzone(deadzone);
    preset.set_deadzone_left(deadzone_left);
    if preset.save() {
        Ok(())
    } else {
        Err("Failed to save preset".to_string())
    }
}


