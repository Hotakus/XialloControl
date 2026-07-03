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
pub mod app_state;


// /// 获取当前鼠标下的窗口句柄
// fn get_window_from_cursor() -> HWND {
//     unsafe {
//         let mut pt = POINT { x: 0, y: 0 };
//         GetCursorPos(&mut pt);
//         WindowFromPoint(pt)
//     }
// }

// /// 初始化 COM 并返回 ITipInvocation 接口
// fn create_tip_invocation() -> Result<ITipInvocation> {
//     unsafe {
//         CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED)?;
//         let tip: ITipInvocation = CoCreateInstance(
//             &TipInvocation,
//             None,
//             CLSCTX_INPROC_SERVER,
//         )?;
//         Ok(tip)
//     }
// }

// /// 根据开关状态控制 TabTip 显示或隐藏
// pub fn toggle_tabtip(enable: bool) -> Result<()> {
//     let hwnd = get_window_from_cursor();
//     let tip = create_tip_invocation()?;

//     unsafe {
//         if enable {
//             tip.Show(hwnd)?;   // 显示键盘
//         } else {
//             tip.Hide(hwnd)?;   // 隐藏键盘
//         }
//     }

//     Ok(())
// }




fn main() {
    // simple_logger::init_with_level(log::Level::Debug).unwrap();
    xeno_utils::initialize();
    xiallocontrol_lib::run();

    // show_virtual_keyboard();

    // #[cfg(target_pointer_width = "64")]
    // println!("我是 64 位程序");

    // #[cfg(target_pointer_width = "32")]
    // println!("我是 32 位程序");

    // loop {
    //     std::thread::sleep(std::time::Duration::from_secs(1));
    // }
}
