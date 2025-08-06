// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod controller;
mod mapping;
mod setting;
mod tray;
mod xeno_utils;
mod preset;
mod setup;


fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    setup::setup();
    xenocontrol_lib::run();
}

