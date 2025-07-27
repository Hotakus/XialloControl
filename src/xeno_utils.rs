use std::env;
use std::path::PathBuf;

pub fn get_app_root() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|exe_path| exe_path.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}


pub fn initialize() {
    println!("Initializing Xeno Utils...");
}