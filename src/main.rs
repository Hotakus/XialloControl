// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::thread;

mod controller;
mod tray;

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    xenocontrol_lib::run();
}