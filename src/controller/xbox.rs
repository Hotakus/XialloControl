#[cfg(target_os = "windows")]
use crate::controller::controller::{disconnect_device, get_app_handle, get_xinput, DeviceInfo, CONTROLLER_DATA};
#[cfg(target_os = "linux")]
use crate::controller::controller::{disconnect_device, DeviceInfo, CONTROLLER_DATA};
use crate::controller::datas::ControllerButtons;
use crate::controller::logic;
use tauri::Emitter;

#[cfg(target_os = "windows")]
use rusty_xinput::XInputState;

/// Xbox控制器状态轮询处理 (Windows)
#[cfg(target_os = "windows")]
fn _poll_xbox_controller_state(state: XInputState) {
    let mut controller_data = CONTROLLER_DATA.write().unwrap();

    // 按钮状态检测
    if state.south_button() {
        println!("Xbox A 键（South）被按下");
        controller_data.set_button(ControllerButtons::A, true);
    } else {
        controller_data.set_button(ControllerButtons::A, false);
    }

    if state.east_button() {
        println!("Xbox B 键（East）被按下");
        controller_data.set_button(ControllerButtons::B, true);
    } else {
        controller_data.set_button(ControllerButtons::B, false);
    }

    if state.north_button() {
        println!("Xbox Y 键（North）被按下");
        controller_data.set_button(ControllerButtons::Y, true);
    } else {
        controller_data.set_button(ControllerButtons::Y, false);
    }

    if state.west_button() {
        println!("Xbox X 键（West）被按下");
        controller_data.set_button(ControllerButtons::X, true);
    } else {
        controller_data.set_button(ControllerButtons::X, false);
    }

    if state.guide_button() {
        println!("Xbox Guide 键被按下");
        controller_data.set_button(ControllerButtons::Guide, true);
    } else {
        controller_data.set_button(ControllerButtons::Guide, false);
    }

    if state.start_button() {
        println!("Xbox Start 键被按下");
        controller_data.set_button(ControllerButtons::Start, true);
    } else {
        controller_data.set_button(ControllerButtons::Start, false);
    }

    if state.select_button() {
        println!("Xbox Select 键被按下");
        controller_data.set_button(ControllerButtons::Back, true);
    } else {
        controller_data.set_button(ControllerButtons::Back, false);
    }

    if state.left_thumb_button() {
        println!("Xbox 左摇杆按下");
        controller_data.left_stick.is_pressed = true;
    } else {
        controller_data.left_stick.is_pressed = false;
    }

    if state.right_thumb_button() {
        println!("Xbox 右摇杆按下");
        controller_data.right_stick.is_pressed = true;
    } else {
        controller_data.right_stick.is_pressed = false;
    }

    // 摇杆状态读取
    let (lx, ly) = state.left_stick_raw();
    let (rx, ry) = state.right_stick_raw();

    // 归一化处理
    controller_data.left_stick.x = logic::normalize(lx, -32768, 32767, -1.0, 1.0).unwrap() as f32;
    controller_data.left_stick.y = logic::normalize(ly, -32768, 32767, -1.0, 1.0).unwrap() as f32;
    controller_data.right_stick.x = logic::normalize(rx, -32768, 32767, -1.0, 1.0).unwrap() as f32;
    controller_data.right_stick.y = logic::normalize(ry, -32768, 32767, -1.0, 1.0).unwrap() as f32;

    // 触发器状态读取
    let lt = state.left_trigger();
    let rt = state.right_trigger();
    let lt_bool = state.left_trigger_bool();
    let rt_bool = state.right_trigger_bool();

    controller_data.left_trigger.value = lt as f32;
    controller_data.right_trigger.value = rt as f32;
    controller_data.left_trigger.is_pressed = lt_bool;
    controller_data.right_trigger.is_pressed = rt_bool;

    controller_data.left_trigger.has_pressure = true;

    let app_handle = get_app_handle();
    app_handle
        .emit("update_controller_data", *controller_data)
        .expect("TODO: panic message");

    // println!("({lx}, {ly}) - ({rx}, {ry}) {}, {}", state.left_trigger(), state.left_trigger_bool());
}

/// Xbox控制器轮询入口 (Windows)
#[cfg(target_os = "windows")]
pub fn poll_xbox_controller(_device: &DeviceInfo) {
    let xinput = get_xinput();
    match xinput.get_state_ex(0).or_else(|_| xinput.get_state(0)) {
        Ok(state) => _poll_xbox_controller_state(state),
        Err(_) => {
            // 控制器断开处理
            disconnect_device();
            let app_handle = get_app_handle();
            if let Err(e) = app_handle.emit("physical_connect_status", false) {
                log::error!("发送 physical_connect_status 事件失败: {e}");
            }
        }
    }
}

/// Xbox控制器轮询入口 (Linux)
#[cfg(target_os = "linux")]
pub fn poll_xbox_controller(_device: &DeviceInfo) {
    println!("poll_xbox_controllers");
}
