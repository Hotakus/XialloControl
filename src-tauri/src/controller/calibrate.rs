use crate::controller::controller::{RAW_CONTROLLER_DATA, CURRENT_DEVICE, DeviceInfo};
use crate::xeno_utils;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const CALIBRATIONS_DIR: &str = "calibrations";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StickCaliMode {
    Circle,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StickTestSteps {
    Idle,
    CenterCheck,
    RangeDetection,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StickRange {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

impl StickRange {
    pub fn new() -> Self {
        Self {
            x_min: f32::MAX,
            x_max: f32::MIN,
            y_min: f32::MAX,
            y_max: f32::MIN,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StickCalibration {
    pub step: StickTestSteps,
    pub mode: StickCaliMode,
    pub stick_center: (f32, f32),
    pub stick_range: StickRange,
}

impl StickCalibration {
    pub fn new() -> StickCalibration {
        StickCalibration {
            step: StickTestSteps::Idle,
            mode: StickCaliMode::Square, // 默认为方形
            stick_center: (0.0, 0.0),
            stick_range: StickRange::new(),
        }
    }

    pub fn reset(&mut self) {
        self.step = StickTestSteps::Idle;
        self.mode = StickCaliMode::Square;
        self.stick_center = (0.0, 0.0);
        self.stick_range.reset();
    }

    pub fn record_center(&mut self, x: f32, y: f32) {
        self.stick_center = (x, y);
    }

    pub fn update_range(&mut self, x: f32, y: f32) {
        self.stick_range.x_min = self.stick_range.x_min.min(x);
        self.stick_range.x_max = self.stick_range.x_max.max(x);
        self.stick_range.y_min = self.stick_range.y_min.min(y);
        self.stick_range.y_max = self.stick_range.y_max.max(y);
    }

    pub fn update_step_to(&mut self, step: StickTestSteps) {
        self.step = step;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ControllerCalibration {
    pub left_stick: StickCalibration,
    pub right_stick: StickCalibration,
}

impl ControllerCalibration {
    pub fn new() -> Self {
        Self {
            left_stick: StickCalibration::new(),
            right_stick: StickCalibration::new(),
        }
    }
    
    pub fn reset(&mut self) {
        self.left_stick.reset();
        self.right_stick.reset();
    }
}

/// 用于序列化到文件的结构
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CalibrationFile {
    pub left_stick_calibration: StickCalibration,
    pub right_stick_calibration: StickCalibration,
}

/// 全局控制器校准数据
pub static CONTROLLER_CALIBRATION: Lazy<RwLock<ControllerCalibration>> =
    Lazy::new(|| RwLock::new(ControllerCalibration::new()));

/// 校准监听线程的句柄
static CALIBRATION_THREAD: Lazy<Mutex<Option<JoinHandle<()>>>> = Lazy::new(|| Mutex::new(None));
/// 线程生命周期控制标志
static IS_CALIBRATING: AtomicBool = AtomicBool::new(false);

// ---------------------- 文件持久化 ----------------------

fn get_calibration_filepath(device: &DeviceInfo) -> Option<PathBuf> {
    let vid = &device.vendor_id;
    let pid = device.product_id.as_deref().unwrap_or("0000");
    let subpid = device.sub_product_id.as_deref().unwrap_or("0000");
    let filename = format!("cali_{}_{}_{}.toml", vid, pid, subpid);
    xeno_utils::ensure_dir(&PathBuf::from(CALIBRATIONS_DIR)).map(|dir| dir.join(filename))
}

pub fn save_calibration(device: &DeviceInfo, cali_data: &ControllerCalibration) -> Result<(), String> {
    if let Some(path) = get_calibration_filepath(device) {
        let file_content = CalibrationFile {
            left_stick_calibration: cali_data.left_stick,
            right_stick_calibration: cali_data.right_stick,
        };
        xeno_utils::write_toml_file(&path, &file_content)
            .map_err(|e| format!("写入校准文件失败: {}", e))
    } else {
        Err("无法获取校准文件路径".to_string())
    }
}

pub fn load_calibration(device: &DeviceInfo) {
    let settings = crate::setting::get_setting();
    let mode = match settings.calibration_mode.as_str() {
        "square" => StickCaliMode::Square,
        _ => StickCaliMode::Circle,
    };

    if let Some(path) = get_calibration_filepath(device) {
        if path.exists() {
            match xeno_utils::read_toml_file::<CalibrationFile>(&path) {
                Ok(file_content) => {
                    let mut cali_data = CONTROLLER_CALIBRATION.write().unwrap();
                    cali_data.left_stick = file_content.left_stick_calibration;
                    cali_data.right_stick = file_content.right_stick_calibration;
                    // 从设置加载校准模式
                    cali_data.left_stick.mode = mode;
                    cali_data.right_stick.mode = mode;
                    log::info!("成功加载设备 {:?} 的校准文件，模式为 {:?}", device.name, mode);
                }
                Err(e) => {
                    log::error!("读取校准文件失败: {}, 将使用默认值", e);
                    reset_calibration();
                }
            }
        } else {
            log::info!("设备 {:?} 没有找到校准文件，使用默认值", device.name);
            reset_calibration();
        }
    }
}

pub fn reset_calibration() {
    CONTROLLER_CALIBRATION.write().unwrap().reset();
}


// ---------------------- 公共 Getters ----------------------
pub fn get_current_calibration() -> ControllerCalibration {
    *CONTROLLER_CALIBRATION.read().unwrap()
}

// ---------------------- Tauri 命令接口 ----------------------
#[tauri::command]
pub fn get_calibration_state() -> ControllerCalibration {
    *CONTROLLER_CALIBRATION.read().unwrap()
}

#[tauri::command]
pub fn save_current_calibration() -> Result<(), String> {
    log::info!("尝试保存当前校准...");
    let device = CURRENT_DEVICE.read().unwrap();
    // 保存的是最终的校准数据，而不是校准过程中的瞬时状态
    let mut cali_data = CONTROLLER_CALIBRATION.write().unwrap();
    cali_data.left_stick.step = StickTestSteps::Idle;
    cali_data.right_stick.step = StickTestSteps::Idle;
    
    save_calibration(&device, &cali_data)
}

#[tauri::command]
pub fn reset_calibration_to_default() -> Result<(), String> {
    log::info!("尝试重置校准为默认值...");
    let device = CURRENT_DEVICE.read().unwrap();

    if let Some(path) = get_calibration_filepath(&device) {
        if path.exists() {
            if let Err(e) = std::fs::remove_file(&path) {
                let err_msg = format!("删除校准文件失败: {}", e);
                log::error!("{}", err_msg);
                return Err(err_msg);
            }
        }
    }
    
    reset_calibration();
    Ok(())
}

#[tauri::command]
pub fn start_stick_calibration(stick_side: &str) {
    let mut thread_handle = CALIBRATION_THREAD.lock().unwrap();

    if let Some(handle) = thread_handle.take() {
        if !handle.is_finished() {
            log::warn!("校准任务已在运行，请先结束当前任务");
            *thread_handle = Some(handle);
            return;
        }
        handle.join().expect("无法 join 已结束的校准线程");
        log::info!("已清理上次的校准线程");
    }

    {
        let mut cali_data = CONTROLLER_CALIBRATION.write().unwrap();
        if stick_side == "left" {
            let stick_to_cali = &mut cali_data.left_stick;
            stick_to_cali.reset();
            stick_to_cali.update_step_to(StickTestSteps::CenterCheck);
        } else {
            let stick_to_cali = &mut cali_data.right_stick;
            stick_to_cali.reset();
            stick_to_cali.update_step_to(StickTestSteps::CenterCheck);
        }
    }
    
    log::info!("开始校准: {} stick", stick_side);
    
    IS_CALIBRATING.store(true, Ordering::SeqCst);
    
    *thread_handle = Some(thread::spawn(calibration_listener));
}

#[tauri::command]
pub fn next_stick_calibration_step(stick_side: &str) {
    if !IS_CALIBRATING.load(Ordering::SeqCst) { return }

    let mut cali_data = CONTROLLER_CALIBRATION.write().unwrap();
    let stick_to_cali = if stick_side == "left" {
        &mut cali_data.left_stick
    } else {
        &mut cali_data.right_stick
    };

    let next_step = match stick_to_cali.step {
        StickTestSteps::CenterCheck => StickTestSteps::RangeDetection,
        StickTestSteps::RangeDetection => StickTestSteps::Complete,
        _ => stick_to_cali.step,
    };
    stick_to_cali.update_step_to(next_step);
    log::info!("下一步校准: {} stick, 步骤: {:?}", stick_side, stick_to_cali.step);
    
    if next_step == StickTestSteps::Complete {
        IS_CALIBRATING.store(false, Ordering::SeqCst);
        log::info!("校准完成，监听线程将在下次循环后自行结束...");
    }
}

#[tauri::command]
pub fn cancel_stick_calibration(stick_side: &str) {
    if !IS_CALIBRATING.load(Ordering::SeqCst) { return }
    
    let mut cali_data = CONTROLLER_CALIBRATION.write().unwrap();
    if stick_side == "left" {
        cali_data.left_stick.reset();
    } else {
        cali_data.right_stick.reset();
    }
    log::info!("取消校准: {} stick", stick_side);
    
    IS_CALIBRATING.store(false, Ordering::SeqCst);
}

#[tauri::command]
pub fn set_calibration_mode(app_handle: tauri::AppHandle, mode: &str) {
    let new_mode = match mode {
        "square" => StickCaliMode::Square,
        _ => StickCaliMode::Circle,
    };

    // 更新当前校准状态
    {
        let mut cali_data = CONTROLLER_CALIBRATION.write().unwrap();
        cali_data.left_stick.mode = new_mode;
        cali_data.right_stick.mode = new_mode;
        log::info!("设置所有摇杆校准模式为: {:?}", new_mode);
    }

    // 更新并保存应用设置
    let mut settings = crate::setting::get_setting();
    settings.calibration_mode = mode.to_string();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::setting::update_settings(app_handle, settings).await {
            log::error!("保存校准模式设置失败: {:?}", e);
        }
    });
}

fn calibration_listener() {
    log::info!("🔬 校准监听任务已启动");
    while IS_CALIBRATING.load(Ordering::SeqCst) {
        let controller_data = RAW_CONTROLLER_DATA.read().unwrap();
        let raw_lx = controller_data.left_stick.x;
        let raw_ly = controller_data.left_stick.y;
        let raw_rx = controller_data.right_stick.x;
        let raw_ry = controller_data.right_stick.y;
        drop(controller_data); 

        let mut cali_data = CONTROLLER_CALIBRATION.write().unwrap();

        match cali_data.left_stick.step {
            StickTestSteps::CenterCheck => cali_data.left_stick.record_center(raw_lx, raw_ly),
            StickTestSteps::RangeDetection => cali_data.left_stick.update_range(raw_lx, raw_ly),
            _ => {}
        }

        match cali_data.right_stick.step {
            StickTestSteps::CenterCheck => cali_data.right_stick.record_center(raw_rx, raw_ry),
            StickTestSteps::RangeDetection => cali_data.right_stick.update_range(raw_rx, raw_ry),
            _ => {}
        }
        
        thread::sleep(Duration::from_millis(2));
    }
    log::info!("校准状态标志为 false，监听任务结束。");
}

// ---------------------- 校准应用逻辑 ----------------------

/// 将一个已归一化但不完美的摇杆输入，通过校准数据，转换为应用了死区的完美输出。
///
/// # Arguments
/// * `normalized_x` - 由 poll 函数提供的、在 [-1, 1] 区间内的 X 轴值
/// * `normalized_y` - 由 poll 函数提供的、在 [-1, 1] 区间内的 Y 轴值
/// * `deadzone_percent` - 死区百分比 (0-100)
/// * `calibration` - `StickCalibration` 结构体引用，包含在归一化空间内记录的校准数据
///
/// # Returns
/// A tuple `(f32, f32)` representing the fully calibrated stick values, ranging from -1.0 to 1.0.
pub fn apply_calibration(
    normalized_x: f32,
    normalized_y: f32,
    deadzone_percent: u8,
    calibration: &StickCalibration,
) -> (f32, f32) {
    // 如果校准数据是初始状态（即从未校准过），则直接返回原始归一化值
    if calibration.stick_range.x_min == f32::MAX || calibration.stick_range.x_max == f32::MIN {
        return (normalized_x, normalized_y);
    }

    // 1. 中心点校正 (在归一化空间内)
    let corrected_x = normalized_x - calibration.stick_center.0;
    let corrected_y = normalized_y - calibration.stick_center.1;

    // 2. 范围重映射 (将记录的物理范围 [min, max] 映射到完美的 [-1, 1] 逻辑范围)
    // 避免除以零
    let range_x_pos = calibration.stick_range.x_max - calibration.stick_center.0;
    let range_x_neg = calibration.stick_center.0 - calibration.stick_range.x_min;
    let range_y_pos = calibration.stick_range.y_max - calibration.stick_center.1;
    let range_y_neg = calibration.stick_center.1 - calibration.stick_range.y_min;

    let scaled_x = if corrected_x > 0.0 {
        if range_x_pos > 0.0 { corrected_x / range_x_pos } else { 0.0 }
    } else if corrected_x < 0.0 {
        if range_x_neg > 0.0 { corrected_x / range_x_neg } else { 0.0 }
    } else {
        0.0
    };

    let scaled_y = if corrected_y > 0.0 {
        if range_y_pos > 0.0 { corrected_y / range_y_pos } else { 0.0 }
    } else if corrected_y < 0.0 {
        if range_y_neg > 0.0 { corrected_y / range_y_neg } else { 0.0 }
    } else {
        0.0
    };

    // 3. 根据校准模式应用不同的死区和塑形
    let deadzone = deadzone_percent as f32 / 100.0;

    let (final_x, final_y) = match calibration.mode {
        StickCaliMode::Circle => {
            // --- 圆形模式 ---
            // 1. 计算到中心的距离
            let distance = (scaled_x.powi(2) + scaled_y.powi(2)).sqrt();
            
            // 2. 应用圆形死区
            if distance < deadzone {
                return (0.0, 0.0);
            }
            
            // 3. 死区补偿与向量重缩放
            // 将向量长度从 [deadzone, 1.0] 重新映射到 [0, 1.0]
            let rescale_factor = (distance - deadzone) / (1.0 - deadzone);
            
            // 4. 保持方向，应用新的长度
            let x = if distance > 0.0 { (scaled_x / distance) * rescale_factor } else { 0.0 };
            let y = if distance > 0.0 { (scaled_y / distance) * rescale_factor } else { 0.0 };
            (x, y)
        }
        StickCaliMode::Square => {
            // --- 方形模式 (最纯粹的轴向处理) ---
            // 1. 定义轴向死区函数
            let apply_axial_deadzone = |val: f32| {
                if val.abs() < deadzone {
                    0.0
                } else {
                    // 死区补偿：将 [deadzone, 1.0] 映射到 [0, 1.0]
                    (val.abs() - deadzone) / (1.0 - deadzone) * val.signum()
                }
            };
            
            // 2. 分别对X轴和Y轴应用死区
            let x = apply_axial_deadzone(scaled_x);
            let y = apply_axial_deadzone(scaled_y);
            
            // 3. 直接返回结果，不进行任何圆形限制或投射
            (x, y)
        }
    };

    // 4. 最终值裁剪
    (
        final_x.clamp(-1.0, 1.0),
        final_y.clamp(-1.0, 1.0),
    )
}
