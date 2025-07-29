// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;

mod controller;
mod setting;
mod tray;
mod xeno_utils;
mod controller_supports;
mod adaptive_sampler;
mod mapping;

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    xeno_utils::create_config_dir();
    xenocontrol_lib::run();
}

