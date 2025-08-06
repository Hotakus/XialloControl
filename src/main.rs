// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::controller::controller::{_disconnect_device, GLOBAL_GILRS, SAMPLING_RATE};
use gilrs::{Axis, Button, Event, EventType, GamepadId, Gilrs};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use hidapi::HidApi;

pub mod adaptive_sampler;
pub mod controller;
pub mod mapping;
pub mod setting;
pub mod tray;
pub mod xeno_utils;
pub mod preset;
pub mod setup;


fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    setup::setup();
    xenocontrol_lib::run();


    // let mut gilrs = Gilrs::new().unwrap();
    //
    // // Iterate over all connected gamepads
    // for (_id, gamepad) in gilrs.gamepads() {
    //     println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    // }
    //
    // let mut active_gamepad = None;
    //
    // loop {
    //     // Examine new events
    //     while let Some(Event { id, event, time, .. }) = gilrs.next_event() {
    //         println!("{:?} New event from {}: {:?}", time, id, event);
    //         active_gamepad = Some(id);
    //     }
    //
    //     // You can also use cached gamepad state
    //     if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
    //         if gamepad.is_pressed(Button::South) {
    //             println!("gamepad {} Button South is pressed (XBox - A, PS - X)", gamepad.name());
    //         }
    //     }
    // }

}
