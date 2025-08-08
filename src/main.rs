// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod adaptive_sampler;
pub mod controller;
pub mod mapping;
pub mod setting;
pub mod tray;
pub mod xeno_utils;
pub mod preset;
pub mod setup;


fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    setup::setup();
    xenocontrol_lib::run();
}
