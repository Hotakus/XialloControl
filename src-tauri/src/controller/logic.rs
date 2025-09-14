use crate::controller::controller::{
    CONTROLLER_DATA, JOYSTICK_ROTATION_STATES, JoystickRotationState, JoystickSource,
};
use crate::controller::datas::{ControllerDatas, ControllerStick, JoystickRotation};
use crate::{controller, preset};
use num_traits::ToPrimitive;
use std::time::Instant;

pub fn normalize<T>(
    value: T,
    source_min: T,
    source_max: T,
    target_min: f64,
    target_max: f64,
) -> Option<f64>
where
    T: ToPrimitive + Copy,
{
    let value_f = value.to_f64()?;
    let source_min_f = source_min.to_f64()?;
    let source_max_f = source_max.to_f64()?;
    let target_min_f = target_min.to_f64()?;
    let target_max_f = target_max.to_f64()?;

    if (source_max_f - source_min_f).abs() < f64::EPSILON {
        return None; // 防止除以零喵
    }

    let normalized = (value_f - source_min_f) / (source_max_f - source_min_f);
    let scaled = normalized * (target_max_f - target_min_f) + target_min_f;

    Some(scaled)
}

pub(crate) fn get_calibrated_stick_values(
    raw_lx: f32,
    raw_ly: f32,
    raw_rx: f32,
    raw_ry: f32,
) -> (f32, f32, f32, f32) {
    let preset = preset::get_current_preset();
    let cali_data = controller::calibrate::get_current_calibration();

    let (lx, ly) = controller::calibrate::apply_calibration(
        raw_lx,
        raw_ly,
        preset.items.deadzone_left,
        &cali_data.left_stick,
    );

    let (rx, ry) = controller::calibrate::apply_calibration(
        raw_rx,
        raw_ry,
        preset.items.deadzone,
        &cali_data.right_stick,
    );
    (lx, ly, rx, ry)
}

/// 通用函数: 计算并更新单个摇杆的旋转状态, 并返回旋转状态
pub fn update_joystick_rotation_state(source: JoystickSource, x: f32, y: f32) -> JoystickRotation {
    const ROTATION_THRESHOLD: f32 = 15.0 * std::f32::consts::PI / 180.0; // 15度的弧度值
    const JUMP_THRESHOLD: f32 = std::f32::consts::PI; // 角度跳变的阈值, 约180度

    let mut rotation_states = JOYSTICK_ROTATION_STATES.write().unwrap();
    let mut state = rotation_states.remove(&source).unwrap_or_default();

    let is_active = x != 0.0 || y != 0.0;
    let mut rotation_this_frame = JoystickRotation::None; // 本帧的旋转结果

    if is_active {
        let angle = (-y).atan2(x);
        if state.was_active {
            let mut delta = angle - state.last_angle;
            // 处理角度从 -PI 到 PI 或反向的跳变
            if delta > JUMP_THRESHOLD {
                delta -= 2.0 * std::f32::consts::PI;
            } else if delta < -JUMP_THRESHOLD {
                delta += 2.0 * std::f32::consts::PI;
            }

            // 忽略因摇杆回弹引起的大幅度跳变
            if delta.abs() < JUMP_THRESHOLD {
                state.accumulated_angle_delta += delta;
            } else {
                state.accumulated_angle_delta = 0.0;
            }

            // 检查累加值是否达到阈值
            if state.accumulated_angle_delta > ROTATION_THRESHOLD {
                rotation_this_frame = JoystickRotation::Clockwise;
                state.accumulated_angle_delta = 0.0; // 触发后重置
            } else if state.accumulated_angle_delta < -ROTATION_THRESHOLD {
                rotation_this_frame = JoystickRotation::CounterClockwise;
                state.accumulated_angle_delta = 0.0; // 触发后重置
            }
        }
        state.last_angle = angle;
    } else {
        // 当摇杆回到死区时, 重置累加器
        state.accumulated_angle_delta = 0.0;
    }

    state.was_active = is_active;
    state.current_rotation = rotation_this_frame;

    rotation_states.insert(source, state);
    rotation_this_frame
}

pub fn apply_deadzone_for_stick(controller_stick: &mut ControllerStick, deadzone: f32) {
    let magnitude =
        (controller_stick.x * controller_stick.x + controller_stick.y * controller_stick.y).sqrt();
    if magnitude < deadzone {
        controller_stick.x = 0.0;
        controller_stick.y = 0.0;
    } else {
        let rescale_factor = (magnitude - deadzone) / (1.0 - deadzone);
        let norm_x = controller_stick.x / magnitude;
        let norm_y = controller_stick.y / magnitude;

        controller_stick.x = norm_x * rescale_factor;
        controller_stick.y = norm_y * rescale_factor;
    }
}

pub fn apply_deadzone(controller_data: &mut ControllerDatas) {
    let preset = preset::get_current_preset();

    let deadzone_left = preset.items.deadzone_left as f32 / 100.0;
    let deadzone_right = preset.items.deadzone as f32 / 100.0;

    apply_deadzone_for_stick(&mut controller_data.left_stick, deadzone_left);
    apply_deadzone_for_stick(&mut controller_data.right_stick, deadzone_right);
}

pub fn check_sticks_rotation(controller_data: &mut ControllerDatas) {
    controller_data.left_stick_rotation = update_joystick_rotation_state(
        JoystickSource::LeftStick,
        controller_data.left_stick.x,
        controller_data.left_stick.y,
    );
    controller_data.right_stick_rotation = update_joystick_rotation_state(
        JoystickSource::RightStick,
        controller_data.right_stick.x,
        controller_data.right_stick.y,
    );
}
