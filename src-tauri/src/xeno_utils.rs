#![allow(dead_code)]

use std::path::PathBuf;
use std::{env, fs};

pub fn get_app_root() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|exe_path| exe_path.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub static GLOBAL_CONFIG_DIR: &str = "xc_datas";

pub fn create_config_dir() {
    let app_root = get_app_root();
    let config_dir = app_root.join(GLOBAL_CONFIG_DIR);
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("创建失败");
    } else {
        log::debug!("Config dir already exists");
    }
}

pub fn get_config_path(file_name: &str) -> PathBuf {
    get_app_root().join(GLOBAL_CONFIG_DIR).join(file_name)
}

pub fn read_toml_file<T: serde::de::DeserializeOwned>(path: &PathBuf) -> anyhow::Result<T> {
    let toml_str = fs::read_to_string(path)?;
    toml::from_str(&toml_str).map_err(|e| e.into())
}

pub fn write_toml_file<T: serde::Serialize>(path: &PathBuf, data: &T) -> anyhow::Result<()> {
    let toml_str = toml::to_string_pretty(data)?;
    fs::write(path, toml_str)?;
    Ok(())
}

pub fn ensure_config_dir() {
    let config_dir = get_app_root().join(GLOBAL_CONFIG_DIR);
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("创建配置目录失败");
    }
}

pub fn ensure_dir(dir: &PathBuf) -> Option<PathBuf> {
    let config_dir = get_app_root().join(GLOBAL_CONFIG_DIR);
    let full_dir = config_dir.join(dir);
    if !full_dir.exists() {
        fs::create_dir_all(&full_dir).expect("创建目录失败");
    }
    Some(full_dir)
}

pub fn initialize() {
    log::debug!("初始化实用工具");
    create_config_dir();
}
