//! 预设数据结构定义
//! 用于预先设计好的预设数据

use serde::{Deserialize, Serialize};

/// 预设映射条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetMappingData {
    /// 手柄按键名称（如 "Square", "A", "R1", "LT" 等）
    pub button: String,
    /// 对应的动作（如 "Control+z", "MouseLeft", "MouseWheelUp" 等）
    pub action: String,
    /// 修饰键列表（如 ["Control", "Shift"]）
    pub modifiers: Vec<String>,
    /// 按键检测模式（"single", "double", "long"）
    pub check_mode: String,
    /// 检测模式参数（双击间隔或长按时间）
    pub check_mode_param: u64,
    /// 触发阈值（主要用于扳机键）
    pub trigger_threshold: f32,
    /// 是否连续触发
    pub continually_trigger: bool,
    /// 触发间隔（毫秒）
    pub interval: u64,
    /// 初始触发间隔（毫秒）
    pub initial_interval: u64,
    /// 最小触发间隔（毫秒）
    pub min_interval: u64,
    /// 加速因子
    pub acceleration: f64,
}

/// 预设配置数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetConfigData {
    /// 右摇杆死区范围 (%)
    pub deadzone: u8,
    /// 左摇杆死区范围 (%)
    pub deadzone_left: u8,
    /// 是否使用摇杆模拟鼠标
    pub use_stick_as_mouse: bool,
    /// 摇杆模拟鼠标配置
    pub stick_as_mouse_simulation: Option<String>,
    /// 鼠标移动速度 (1-100)
    pub move_speed: u8,
    /// 摇杆旋转触发阈值
    pub stick_rotate_trigger_threshold: i16,
    /// 副预设名称
    pub sub_preset_name: Option<String>,
    /// 副预设切换键
    pub sub_preset_switch_button: Option<String>,
    /// 副预设切换模式
    pub sub_preset_switch_mode: Option<String>,
}

/// 完整的预设数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetData {
    /// 预设名称
    pub name: String,
    /// 预设描述
    pub description: Option<String>,
    /// 映射列表
    pub mappings: Vec<PresetMappingData>,
    /// 预设配置
    pub preset_config: PresetConfigData,
}