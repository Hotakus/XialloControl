use crate::controller::controller::{ADAPTER, CONTROLLER_DATA, CURRENT_DEVICE, TIME_INTERVAL};
use crate::controller::datas::{
    ControllerDatas, ControllerLimits, ControllerStick, ControllerTrigger,
};
use num_traits::ToPrimitive;
use std::time::{Duration, Instant};
use tokio::time::interval;

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

const SAMPLING_TIME_INTERVAL_FACTOR: f32 = 1000.0;
const TEMP_INTERVAL: f32 = 1.0 / SAMPLING_TIME_INTERVAL_FACTOR;
const SAMPLING_TIME: f32 = 3.0;

fn average_controller_datas(data_slice: &[ControllerDatas]) -> ControllerDatas {
    let count = data_slice.len() as f32;
    if count == 0.0 {
        return ControllerDatas::new();
    }

    let mut sum_left_x = 0.0f32;
    let mut sum_left_y = 0.0f32;
    let mut sum_right_x = 0.0f32;
    let mut sum_right_y = 0.0f32;

    let mut sum_left_trigger = 0.0f32;
    let mut sum_right_trigger = 0.0f32;

    for data in data_slice {
        sum_left_x += data.left_stick.x;
        sum_left_y += data.left_stick.y;
        sum_right_x += data.right_stick.x;
        sum_right_y += data.right_stick.y;

        sum_left_trigger += data.left_trigger.value;
        sum_right_trigger += data.right_trigger.value;
    }

    let left_stick_center = ((sum_left_x / count).round(), (sum_left_y / count).round());
    let right_stick_center = ((sum_right_x / count).round(), (sum_right_y / count).round());

    ControllerDatas {
        buttons: 0, // 忽略按钮

        left_stick: ControllerStick {
            x: left_stick_center.0,
            y: left_stick_center.1,
            is_pressed: false,
        },
        right_stick: ControllerStick {
            x: right_stick_center.0,
            y: right_stick_center.1,
            is_pressed: false,
        },
        left_trigger: ControllerTrigger {
            value: (sum_left_trigger / count).round(),
            has_pressure: false,
            is_pressed: false,
        },
        right_trigger: ControllerTrigger {
            value: (sum_right_trigger / count).round(),
            has_pressure: false,
            is_pressed: false,
        },

        left_stick_center,
        right_stick_center,

        limits: ControllerLimits::default(),
    }
}

#[tauri::command]
pub async fn controller_stick_drift_sampling() {
    let sampling_time = SAMPLING_TIME;
    let sampling_rate = {
        let adaptive_sampler = ADAPTER.lock().unwrap();
        adaptive_sampler.compute_sampling_rate(TEMP_INTERVAL as f64)
    };

    let original_time_interval = *TIME_INTERVAL.read().unwrap();

    let sample_count = (sampling_time / TEMP_INTERVAL) as usize;
    let mut collected_data = Vec::with_capacity(sample_count);

    log::info!("controller_stick_drift_sampling (tokio async)");

    {
        let mut time_interval = TIME_INTERVAL.write().unwrap();
        *time_interval = TEMP_INTERVAL;
    }

    let mut interval = interval(Duration::from_secs_f32(TEMP_INTERVAL));
    let start = Instant::now();

    tokio::time::sleep(Duration::from_secs_f32(1.1)).await;

    for _ in 0..=sample_count {
        let data = {
            let controller_data = CONTROLLER_DATA.read().unwrap();
            *controller_data
        };
        collected_data.push(data);
        interval.tick().await;
    }

    {
        let mut time_interval = TIME_INTERVAL.write().unwrap();
        *time_interval = original_time_interval;
    }

    let avg = average_controller_datas(&collected_data);
    log::info!(
        "avg stick & trigger: {:#?} , count: {}",
        avg,
        collected_data.len()
    );
}

#[tauri::command]
pub fn check_controller_deadzone() -> (f32, f32) {
    let device = CURRENT_DEVICE.read().unwrap();
    let datas = CONTROLLER_DATA.read().unwrap();
    let mut deadzone: (f32, f32) = (0.0, 0.0);

    let left_stick_max = datas.left_stick.x.abs().max(datas.left_stick.y.abs());
    let right_stick_max = datas.right_stick.x.abs().max(datas.right_stick.y.abs());

    (left_stick_max, right_stick_max)
}
