use crate::xeno_utils;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::{RwLock};
use crate::controller::datas::ControllerDatas;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MappingType {
    Keyboard,
    Controller,
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
    load_mappings(); // 启动时加载映射配置
}



pub fn map(controller_datas: ControllerDatas) {
    
}
