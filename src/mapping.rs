use crate::xeno_utils;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

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
    fn new(
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

    fn get_id(&self) -> u64 {
        self.id
    }

    fn get_controller_button(&self) -> &str {
        &self.composed_button
    }

    fn get_composed_key(&self) -> &str {
        &self.composed_shortcut_key
    }

    fn get_mapping_type(&self) -> MappingType {
        self.mapping_type.clone()
    }
}

// 包装结构体用于文件序列化
#[derive(Serialize, Deserialize)]
struct MappingFile {
    mappings: Vec<Mapping>,
}

pub static GLOBAL_MAPPING_CACHE: Lazy<Mutex<Vec<Mapping>>> = Lazy::new(|| {
    let mappings = vec![];
    Mutex::new(mappings)
});

const MAPPINGS_FILE: &str = "mappings.toml";

pub fn load_mappings() {
    let mappings_path = xeno_utils::get_config_path(MAPPINGS_FILE);

    if !mappings_path.exists() {
        log::warn!("映射配置文件不存在，将创建空文件");
        save_mappings(); // 创建空文件
        return;
    }

    match xeno_utils::read_toml_file::<MappingFile>(&mappings_path) {
        Ok(mapping_file) => {
            let mut cache = GLOBAL_MAPPING_CACHE.lock().unwrap();
            *cache = mapping_file.mappings;
            log::info!("成功加载 {} 条映射配置", cache.len());
        }
        Err(e) => {
            log::error!("加载映射配置失败: {}", e);
        }
    }
}

pub fn save_mappings() {
    // 确保配置目录存在
    xeno_utils::ensure_config_dir();

    // 克隆数据时限制锁的作用域
    let mappings = {
        let cache = GLOBAL_MAPPING_CACHE.lock().unwrap();
        cache.clone()
    };

    let mappings_path = xeno_utils::get_config_path(MAPPINGS_FILE);

    // 使用包装结构体
    let mapping_file = MappingFile {
        mappings: mappings.clone(),
    };

    match xeno_utils::write_toml_file(&mappings_path, &mapping_file) {
        Ok(_) => log::info!("映射配置已保存到: {:?}", mappings_path),
        Err(e) => log::error!("保存映射配置失败: {:#?}", e), // 增强错误日志
    }
}

#[tauri::command]
pub fn set_mapping(mapping: Vec<Mapping>) {
    log::debug!("更新映射配置: {:#?}", mapping);
    {
        // 限制锁的作用域
        let mut mappings = GLOBAL_MAPPING_CACHE.lock().unwrap();
        *mappings = mapping;
    } // 锁在这里自动释放
    log::debug!("映射缓存已更新");
    save_mappings();
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
    load_mappings();
    let mappings = GLOBAL_MAPPING_CACHE.lock().unwrap();
    mappings.clone()
}

pub fn initialize() {
    log::debug!("初始化映射模块");
    load_mappings(); // 启动时加载映射配置
}
