cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        use crate::controller::controller::{get_app_handle, get_xinput, DeviceInfo, CONTROLLER_DATA};
    } else if #[cfg(target_os = "linux")] {
        use crate::controller::controller::{disconnect_device, DeviceInfo, CONTROLLER_DATA};
    }
}
use crate::controller::datas::{ControllerButtons, ControllerDatas};
use crate::controller::logic;
use tauri::Emitter;

use crate::controller::controller::_disconnect_device;
#[cfg(target_os = "windows")]
use rusty_xinput::XInputState;

const MAX_XINPUT_DEVICES: usize = 4;

/// Xbox控制器状态轮询处理 (Windows)
#[cfg(target_os = "windows")]
fn _poll_xbox_controller_state(state: XInputState) {
    let mut controller_data = ControllerDatas::new();

    // 按钮状态检测
    let buttons = [
        (
            state.south_button(),
            ControllerButtons::South,
            "Xbox A 键（South）",
        ),
        (
            state.east_button(),
            ControllerButtons::East,
            "Xbox B 键（East）",
        ),
        (
            state.north_button(),
            ControllerButtons::North,
            "Xbox Y 键（North）",
        ),
        (
            state.west_button(),
            ControllerButtons::West,
            "Xbox X 键（West）",
        ),
        (
            state.guide_button(),
            ControllerButtons::Guide,
            "Xbox Guide 键",
        ),
        (
            state.start_button(),
            ControllerButtons::Start,
            "Xbox Start 键",
        ),
        (
            state.select_button(),
            ControllerButtons::Back,
            "Xbox Select 键",
        ),
        (
            state.arrow_down(),
            ControllerButtons::Down,
            "Xbox 方向键（Down）",
        ),
        (
            state.arrow_left(),
            ControllerButtons::Left,
            "Xbox 方向键（Left）",
        ),
        (
            state.arrow_right(),
            ControllerButtons::Right,
            "Xbox 方向键（Right）",
        ),
        (state.arrow_up(), ControllerButtons::Up, "Xbox 方向键（Up）"),
        (state.left_shoulder(), ControllerButtons::LB, "Xbox LB 键"),
        (state.right_shoulder(), ControllerButtons::RB, "Xbox RB 键"),
        (
            state.left_thumb_button(),
            ControllerButtons::LStick,
            "Xbox 左摇杆按键",
        ),
        (
            state.right_thumb_button(),
            ControllerButtons::RStick,
            "Xbox 右摇杆按键",
        ),
    ];

    for (pressed, button, label) in buttons {
        if pressed {
            log::debug!("{label} 被按下");
        }
        controller_data.set_button(button, pressed);
    }

    controller_data.left_stick.is_pressed = state.left_thumb_button();
    controller_data.right_stick.is_pressed = state.right_thumb_button();

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

    let mut global_controller_data = CONTROLLER_DATA.write().unwrap();
    if *global_controller_data != controller_data {
        *global_controller_data = controller_data;

        let app_handle = get_app_handle();
        app_handle
            .emit("update_controller_data", controller_data)
            .expect("TODO: panic message");
    }
    // println!("({lx}, {ly}) - ({rx}, {ry}) {}, {}", state.left_trigger(), state.left_trigger_bool());
}

/// Xbox控制器轮询入口 (Windows)
#[cfg(target_os = "windows")]
pub fn poll_xbox_controller(_device: &DeviceInfo) {
    let xinput = get_xinput();
    let mut got_device = false;

    for i in 0..MAX_XINPUT_DEVICES {
        match xinput
            .get_state_ex(i as u32)
            .or_else(|_| xinput.get_state(i as u32))
        {
            Ok(state) => {
                let xinput_caps_ex = xinput.get_capabilities_ex(i as u32).unwrap();
                let vid = format!("{:04x}", xinput_caps_ex.vendor_id);
                let pid = format!("{:04x}", xinput_caps_ex.product_id);

                if vid.eq_ignore_ascii_case(&_device.vendor_id)
                    && pid.eq_ignore_ascii_case(_device.product_id.as_deref().unwrap())
                {
                    got_device = true;
                    _poll_xbox_controller_state(state);
                    break;
                }
            }
            Err(_) => {
                got_device = false;
            }
        }
    }

    if !got_device {
        // 控制器断开处理
        log::error!("Xbox 控制器断开连接");
        _disconnect_device();
    }
}

/// Xbox控制器轮询入口 (Linux)
#[cfg(target_os = "linux")]
pub fn poll_xbox_controller(_device: &DeviceInfo) {
    println!("poll_xbox_controllers");
}
