#![allow(dead_code)]

use crate::mapping::{Mapping};
use crate::setting::{get_setting};
use crate::xeno_utils::ensure_dir;
use crate::{mapping, xeno_utils};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

const PRESET_DIR: &str = "presets";
const DEFAULT_PRESET_NAME: &str = "default";
const DEFAULT_MAPPINGS_FILE: &str = "mappings.toml";

const DEFAULT_DEADZONE: u8 = 5; // 10%

pub static CURRENT_PRESET: Lazy<RwLock<Preset>> =
    Lazy::new(|| RwLock::new(Preset::new(DEFAULT_PRESET_NAME.to_string(), vec![])));

pub static CURRENT_PRESET_LIST: Lazy<RwLock<Vec<Preset>>> = Lazy::new(|| RwLock::new(vec![]));

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


/// 创建新的预设
#[tauri::command]
pub fn create_preset(name: &str) -> Result<Preset, String> {
    if name.is_empty() {
        return Err("预设名称不能为空".to_string());
    }

    // 创建，然后返回最新
    let preset = Preset::new(name.to_string(), vec![]);
    let mut presets = CURRENT_PRESET_LIST.write().unwrap();
    if presets.iter().any(|p| p.name == name) {
        Err("预设名称已存在".to_string())
    } else if preset.save() {
        presets.push(preset.clone());
        Ok(preset)
    } else {
        Err("创建预设失败".to_string())
    }
}

/// 删除预设
#[tauri::command]
pub fn delete_preset(name: &str) -> Result<(), String> {
    if name == DEFAULT_PRESET_NAME {
        return Err("不能删除默认预设".to_string());
    }

    let preset_dir = ensure_dir(&PathBuf::from(PRESET_DIR));
    if let Some(preset_path) = preset_dir {
        let target_dir = preset_path.join(name);
        if !target_dir.exists() {
            return Err(format!("预设 {target_dir:#?} 不存在"));
        }

        if let Err(e) = fs::remove_dir_all(&target_dir) {
            log::error!("删除预设失败：{e}");
            return Err(format!("删除预设失败：{e}"));
        }

        // 从全局列表中移除
        let mut presets = CURRENT_PRESET_LIST.write().unwrap();
        presets.retain(|p| p.name != name);
    }
    Ok(())
}

/// 重命名预设
#[tauri::command]
pub fn rename_preset(old_name: &str, new_name: &str) -> Result<(), String> {
    if old_name == new_name {
        return Ok(());
    }

    if new_name.is_empty() {
        return Err("新预设名称不能为空".to_string());
    }

    if old_name == DEFAULT_PRESET_NAME {
        return Err("不能重命名默认预设".to_string());
    }

    let presets_dir = ensure_dir(&PathBuf::from(PRESET_DIR));
    if let Some(presets_path) = presets_dir {
        let old_dir = presets_path.join(old_name);
        let new_dir = presets_path.join(new_name);

        if !old_dir.exists() {
            return Err("原预设不存在".to_string());
        }

        if new_dir.exists() {
            return Err("新预设名称已存在".to_string());
        }

        if let Err(e) = fs::rename(&old_dir, &new_dir) {
            log::error!("重命名预设失败：{e}");
            return Err("重命名预设失败".to_string());
        }

        // 在新目录中，将旧的 .toml 文件重命名为新的 .toml 文件
        let old_toml_path = new_dir.join(old_name.to_string() + ".toml");
        let new_toml_path = new_dir.join(new_name.to_string() + ".toml");

        if old_toml_path.exists() {
            if let Err(e) = fs::rename(&old_toml_path, &new_toml_path) {
                log::error!("重命名预设 .toml 文件失败：{e}");
                // 尝试回滚文件夹重命名操作
                let _ = fs::rename(&new_dir, &old_dir);
                return Err("重命名预设的 .toml 文件失败".to_string());
            }
        } else {
            // 如果旧的 toml 不存在，这是一个严重错误，回滚
            let _ = fs::rename(&new_dir, &old_dir);
            return Err("找不到原始预设的 .toml 文件".to_string());
        }

        // 读取新的 .toml 文件，修改其内部的 name 字段，然后保存
        let updated_preset = match xeno_utils::read_toml_file::<PresetFile>(&new_toml_path) {
            Ok(mut preset_file) => {
                preset_file.preset.name = new_name.to_string();
                if let Err(e) = xeno_utils::write_toml_file(&new_toml_path, &preset_file) {
                    log::error!("更新预设文件内容失败：{e}");
                    return Err("更新预设文件内容失败".to_string());
                }
                preset_file.preset
            }
            Err(e) => {
                log::error!("读取新预设文件失败：{e}");
                return Err("读取新预设文件失败".to_string());
            }
        };

        // 更新全局预设列表
        let mut presets = CURRENT_PRESET_LIST.write().unwrap();
        if let Some(p) = presets.iter_mut().find(|p| p.name == old_name) {
            *p = updated_preset;
            Ok(())
        } else {
            // 如果在列表中找不到，可能是一个不一致的状态，但文件操作已完成
            // 最好也记录一个警告
            log::warn!("文件系统中的预设已重命名，但在全局列表中未找到: {old_name}");
            Ok(())
        }
    } else {
        Err("预设目录不可用".to_string())
    }
}

/// 切换到指定预设
#[tauri::command]
pub fn switch_to_preset(name: &str) -> Result<Preset, String> {
    let mut preset = CURRENT_PRESET.write().unwrap();
    if preset.load(name) {
        // 加载映射文件
        mapping::set_mapping_file_path(
            PathBuf::from(PRESET_DIR)
                .join(preset.name.clone())
                .join(preset.items.mappings_file_name.clone()),
        );
        log::info!("成功切换到预设: {}", mapping::get_mapping_file_path().display());
        Ok(preset.clone())
    } else {
        Err("加载预设失败".to_string())
    }
}

#[tauri::command]
pub fn load_preset(name: &str) -> Preset {
    let mut preset = CURRENT_PRESET.write().unwrap();
    preset.load(name);
    preset.clone()
}

/// 获取所有预设名称列表
#[tauri::command]
pub fn check_presets_list() -> Vec<String> {
    let preset_dir = ensure_dir(&PathBuf::from(PRESET_DIR));
    let mut preset_list = Vec::new();
    if let Some(dir) = preset_dir {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|ft| ft.is_dir()) {
                    if let Some(preset_name) = entry.file_name().to_str() {
                        preset_list.push(preset_name.to_string());
                    }
                }
            }
        }
    }
    log::debug!("Loaded presets: {preset_list:#?}");
    preset_list
}

/// 加载预设列表到全局
pub fn load_presets_from_list_to_global(list: Vec<String>) {
    let mut presets = Vec::new();
    let len = list.len();

    if len == 0 {
        log::info!("预设列表为空");
        return;
    }

    for name in list {
        let mut preset = Preset::new(name.clone(), vec![]);
        if preset.load(name.as_str()) {
            presets.push(preset);
        }
    }
    let mut preset_list = CURRENT_PRESET_LIST.write().unwrap();
    *preset_list = presets.clone();

    log::info!("成功加载预设: {preset_list:#?}");
}

pub fn get_current_preset() -> Preset {
    CURRENT_PRESET.read().unwrap().clone()
}

pub fn initialize() {
    log::info!("初始化预设");

    // 加载上一个预设
    let prev_preset_name = { get_setting().previous_preset.clone() };
    if !prev_preset_name.is_empty() {
        let _ = switch_to_preset(prev_preset_name.as_str());
    } else {
        let _ = switch_to_preset(DEFAULT_PRESET_NAME);
    }

    // 获取目录中的所有映射
    let list = check_presets_list();
    load_presets_from_list_to_global(list);
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
