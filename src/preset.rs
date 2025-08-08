#![allow(dead_code)]

use crate::mapping::{get_mappings, Mapping};
use crate::xeno_utils;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::RwLock;
use once_cell::sync::Lazy;
use crate::setting::load_settings;

const PRESET_DIR: &str = "presets";

const DEFAULT_DEADZONE: u8 = 10; // 10%

pub static CURRENT_PRESET: Lazy<RwLock<Preset>> = Lazy::new(|| {
    RwLock::new(Preset::new("default".to_string(), vec![]))
});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PresetItems {
    pub mappings: Vec<Mapping>,

    /// 右摇杆死区范围 (%)
    #[serde(default = "default_deadzone")]
    pub deadzone: u8,

    /// 左摇杆死区范围 (%)
    #[serde(default = "default_deadzone")]
    pub deadzone_left: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Preset {
    name: String,
    pub items: PresetItems,
}

impl Preset {
    pub fn new(name: String, mappings: Vec<Mapping>) -> Self {
        Self {
            name,
            items: PresetItems {
                mappings,
                deadzone: DEFAULT_DEADZONE,
                deadzone_left: DEFAULT_DEADZONE,
            },
        }
    }

    pub fn save(&self) -> bool {
        let config_dir = xeno_utils::get_config_path(PRESET_DIR);
        if let Err(e) = fs::create_dir_all(&config_dir) {
            log::error!("创建预设目录失败：{e}");
            return false;
        }

        let save_path = config_dir.join(self.name.clone() + ".toml");
        let save_data = PresetFile {
            preset: self.clone(),
        };

        if let Err(e) = xeno_utils::write_toml_file(&save_path, &save_data) {
            log::error!("保存预设失败：{e}");
            return false;
        }

        true
    }

    pub fn load(&mut self, name: &str) {
        let config_dir = xeno_utils::get_config_path(PRESET_DIR);
        let load_path = config_dir.join(name.to_string() + ".toml");

        let load_data = xeno_utils::read_toml_file::<PresetFile>(&load_path);
        match load_data {
            Ok(loaded_preset) => {
                self.name = loaded_preset.preset.name;
                self.items = loaded_preset.preset.items;
            }
            Err(e) => {
                log::error!("加载预设失败：{e}");
            }
        }
    }

    pub fn set_mappings(&mut self, mappings: Vec<Mapping>) {
        self.items.mappings = mappings;
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
    println!("{:#?}", preset.items.mappings);
}

// pub fn load_presets_list() -> Vec<Preset> {
//
// }

#[tauri::command]
pub fn get_preset(name: &str) -> Preset {
    let mut preset = CURRENT_PRESET.write().unwrap();
    preset.load(name);
    preset.clone()
}

