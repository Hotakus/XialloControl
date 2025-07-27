// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod controller;
mod tray;
mod setting;
mod xeno_utils;

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    xenocontrol_lib::run();
}

// fn main() {
//     // 动态加载 XInput DLL
//     // if dynamic_load_xinput().is_err() {
//     //     eprintln!("无法加载 XInput DLL");
//     //     return;
//     // }
//
//     let xinput = XInputHandle::load_default().unwrap();
//
//     loop {
//         // 尝试读取控制器 0（最多支持 0–3）
//         match xinput.get_state(0) {
//             Ok(state) => {
//                 // 象征性使用 Rust 风格的方法判断按钮
//                 if state.south_button() {
//                     println!("Xbox A 键（South）被按下");
//                 }
//                 if state.east_button() {
//                     println!("Xbox B 键（East）被按下");
//                 }
//                 if state.north_button() {
//                     println!("Xbox Y 键（North）被按下");
//                 }
//                 if state.west_button() {
//                     println!("Xbox X 键（West）被按下");
//                 }
//
//                 // 摇杆坐标
//                 let (lx, ly) = state.left_stick_raw();
//                 println!("左摇杆 raw = ({}, {})", lx, ly);
//                 let (rx, ry) = state.right_stick_raw();
//                 println!("右摇杆 raw = ({}, {})", rx, ry);
//             }
//             Err(err) => {
//                 println!("手柄未连接或无法读取状态: {:?}", err);
//             }
//         }
//
//         thread::sleep(Duration::from_millis(500));
//     }
// }

// fn main() {
//     let mut gilrs = Gilrs::new().expect("无法初始化 gilrs");
//
//     println!("🎮 gilrs 0.11 已初始化");
//
//     // 列出当前连接的手柄
//     for (_id, gamepad) in gilrs.gamepads() {
//         println!(
//             "❇️ 检测到手柄 {}: {} (VID={:?}, PID={:?})",
//             _id,
//             gamepad.name(),
//             gamepad.vendor_id(),
//             gamepad.product_id()
//         );
//     }
//
//     println!("\n开始监听输入事件...\n");
//
//     loop {
//         while let Some(Event { id, event, .. }) = gilrs.next_event() {
//             match event {
//                 EventType::ButtonPressed(button, _) => {
//                     println!("🎯 手柄 {} 按下按钮 {:?}", id, button);
//                 }
//                 EventType::ButtonReleased(button, _) => {
//                     println!("🔄 手柄 {} 释放按钮 {:?}", id, button);
//                 }
//                 EventType::AxisChanged(axis, value, _) => {
//                     if value.abs() > 0.1 {
//                         println!("🧭 手柄 {} 轴 {:?} 值 {:.2}", id, axis, value);
//                     }
//                 }
//                 evt => {
//                     println!("ℹ️ 其他事件: {:?}", evt);
//                 }
//             }
//         }
//     }
// }

