cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        use crate::controller::controller::{self, get_xinput, DeviceInfo, CONTROLLER_DATA, JoystickSource};
    } else if #[cfg(target_os = "linux")] {
        use crate::controller::controller::{self, disconnect_device, DeviceInfo, CONTROLLER_DATA, RAW_CONTROLLER_DATA, JoystickSource};
    }
}
use crate::controller::datas::ControllerButtons;
use crate::controller::logic;

use crate::controller::controller::{pack_and_send_data, physical_disconnect_device};
#[cfg(target_os = "windows")]
use rusty_xinput::XInputState;

const MAX_XINPUT_DEVICES: usize = 4;

/// Xbox控制器状态轮询处理 (Windows)
#[cfg(target_os = "windows")]
fn _poll_xbox_controller_state(state: XInputState) {
    // let mut controller_data = ControllerDatas::new();
    let mut controller_data = CONTROLLER_DATA.write().unwrap();

    // 按钮状态检测
    let buttons = [
        (state.south_button(), ControllerButtons::South, "Xbox A 键（South）"),
        (state.east_button(), ControllerButtons::East, "Xbox B 键（East）"),
        (state.north_button(), ControllerButtons::North, "Xbox Y 键（North）"),
        (state.west_button(), ControllerButtons::West, "Xbox X 键（West）"),
        (state.guide_button(), ControllerButtons::Guide, "Xbox Guide 键"),
        (state.start_button(), ControllerButtons::Start, "Xbox Start 键"),
        (state.select_button(), ControllerButtons::Back, "Xbox Select 键"),
        (state.arrow_down(), ControllerButtons::Down, "Xbox 方向键（Down）"),
        (state.arrow_left(), ControllerButtons::Left, "Xbox 方向键（Left）"),
        (state.arrow_right(), ControllerButtons::Right, "Xbox 方向键（Right）"),
        (state.arrow_up(), ControllerButtons::Up, "Xbox 方向键（Up）"),
        (state.left_shoulder(), ControllerButtons::LB, "Xbox LB 键"),
        (state.right_shoulder(), ControllerButtons::RB, "Xbox RB 键"),
        (state.left_thumb_button(), ControllerButtons::LStick, "Xbox 左摇杆按键"),
        (state.right_thumb_button(), ControllerButtons::RStick, "Xbox 右摇杆按键"),
    ];


    for (pressed, button, label) in buttons {
        controller_data.set_button(button, pressed);
    }

    controller_data.left_stick.is_pressed = state.left_thumb_button();
    controller_data.right_stick.is_pressed = state.right_thumb_button();

    // 摇杆状态读取
    let (lx, ly) = state.left_stick_raw();
    let (rx, ry) = state.right_stick_raw();

    // 归一化处理
    let raw_lx = logic::normalize(lx, -32768, 32767, -1.0, 1.0).unwrap_or(0.0) as f32;
    let raw_ly = logic::normalize(ly, -32768, 32767, -1.0, 1.0).unwrap_or(0.0) as f32;
    let raw_rx = logic::normalize(rx, -32768, 32767, -1.0, 1.0).unwrap_or(0.0) as f32;
    let raw_ry = logic::normalize(ry, -32768, 32767, -1.0, 1.0).unwrap_or(0.0) as f32;

    // // 将原始数据写入 RAW_CONTROLLER_DATA 供校准线程使用
    // {
    //     let mut raw_data = crate::controller::controller::RAW_CONTROLLER_DATA.write().unwrap();
    //     raw_data.left_stick.x = raw_lx;
    //     raw_data.left_stick.y = raw_ly;
    //     raw_data.right_stick.x = raw_rx;
    //     raw_data.right_stick.y = raw_ry;
    // }


    controller_data.left_stick.x = raw_lx;
    controller_data.left_stick.y = raw_ly;
    controller_data.right_stick.x = raw_rx;
    controller_data.right_stick.y = raw_ry;

    // 触发器状态读取
    let lt = state.left_trigger();
    let rt = state.right_trigger();

    controller_data.left_trigger.value = logic::normalize(lt, 0, 255, 0.0, 1.0).unwrap() as f32;
    controller_data.right_trigger.value = logic::normalize(rt, 0, 255, 0.0, 1.0).unwrap() as f32;
    // controller_data.left_trigger.check_triggered(None);
    // controller_data.right_trigger.check_triggered(None);
    // let (l, r) = (controller_data.left_trigger.is_triggered(), controller_data.right_trigger.is_triggered());
    // controller_data.set_button(ControllerButtons::LT, l);
    // controller_data.set_button(ControllerButtons::RT, r);

    // log::error!("({}-{})", controller_data.left_trigger.value, controller_data.right_trigger.value);

    controller_data.left_trigger.has_pressure = true;

    pack_and_send_data(&controller_data);
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
                let spid = format!("{:04x}", xinput_caps_ex.revision_id);
                

                // log::warn!("{vid:#?}/{pid:#?}/{spid:#?} - {:#?}/{:#?}/{:#?} - {:#?}",
                //     _device.vendor_id,
                //     _device.product_id.as_deref().unwrap(),
                //     _device.sub_product_id.as_deref().unwrap(), 
                //     xinput_caps_ex.capabilities.SubType);

                let d_vid = &_device.vendor_id;
                let d_pid = &_device.product_id.as_deref().unwrap();
                let d_sub_pid = &_device.sub_product_id.as_deref().unwrap();

                if vid.eq_ignore_ascii_case(d_vid)
                    && (pid.eq_ignore_ascii_case(d_pid) || pid.eq_ignore_ascii_case(d_sub_pid))
                {
                    got_device = true;
                    _poll_xbox_controller_state(state);
                    break;
                } else {
                    // 错误
                    log::error!("Xbox 控制器连接错误，数据不匹配 ({vid:#?}/{pid:#?}) - ({:#?}/{:#?})",
                        _device.vendor_id,
                        _device.sub_product_id.as_deref().unwrap()
                    );
                }
            }
            Err(_) => {
                log::error!("Xbox 控制器连接错误，设备索引 {i} 不存在");
                got_device = false;
            }
        }
    }

    if !got_device {
        // 控制器断开处理
        log::warn!("Xbox 控制器断开连接");
        physical_disconnect_device();
    }
}

/// Xbox控制器轮询入口 (Linux)
#[cfg(target_os = "linux")]
pub fn poll_xbox_controller(_device: &DeviceInfo) {
    println!("poll_xbox_controllers");
}
