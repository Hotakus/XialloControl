// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod adaptive_sampler;
pub mod controller;
pub mod mapping;
pub mod preset;
pub mod setting;
pub mod setup;
pub mod tray;
pub mod xeno_utils;

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    setup::setup();
    xenocontrol_lib::run();
}
