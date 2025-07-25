// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::thread;

mod controller;
mod tray;

fn main() {

    let config = controller::load_or_create_config("devices.json");
    let devices = controller::list_supported_connected_devices(&config);

    println!("找到以下支持的设备：");
    for (i, d) in devices.iter().enumerate() {
        println!("{}: {} (VID: {}, PID: {})", i, d.name, d.vendor_id, d.product_id);
    }



    // init logger
    simple_logger::init_with_level(log::Level::Info).unwrap();


    // init
    let mut threads_list: Vec<thread::JoinHandle<()>> = vec![];

    //tray::initialize();

    xenocontrol_lib::run();

}