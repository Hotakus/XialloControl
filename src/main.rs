// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod controller;
mod setting;
mod tray;
mod xeno_utils;
mod controller_supports;
mod adaptive_sampler;

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    xenocontrol_lib::run();
}

