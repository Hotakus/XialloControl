//! PS4 手柄预设数据定义

use crate::preset::preset_data::{PresetData, PresetMappingData, PresetConfigData};

/// FPS 游戏 PS4 预设数据
pub fn get_fps_preset_data() -> PresetData {
    PresetData {
        name: "FPS游戏预设".to_string(),
        description: Some("适用于第一人称射击游戏的 PS4 按键映射".to_string()),
        mappings: vec![
            PresetMappingData {
                button: "R2".to_string(),
                action: "MouseLeft".to_string(),
                modifiers: vec![],
                check_mode: "single".to_string(),
                check_mode_param: 300,
                trigger_threshold: 0.3,
                continually_trigger: false,
                interval: 300,
                initial_interval: 300,
                min_interval: 100,
                acceleration: 0.8,
            },
            PresetMappingData {
                button: "L2".to_string(),
                action: "MouseRight".to_string(),
                modifiers: vec![],
                check_mode: "single".to_string(),
                check_mode_param: 300,
                trigger_threshold: 0.3,
                continually_trigger: false,
                interval: 300,
                initial_interval: 300,
                min_interval: 100,
                acceleration: 0.8,
            },
            PresetMappingData {
                button: "Square".to_string(),
                action: "Control+z".to_string(),
                modifiers: vec!["Control".to_string()],
                check_mode: "single".to_string(),
                check_mode_param: 300,
                trigger_threshold: 0.3,
                continually_trigger: false,
                interval: 300,
                initial_interval: 300,
                min_interval: 100,
                acceleration: 0.8,
            },
            PresetMappingData {
                button: "Triangle".to_string(),
                action: "Control+y".to_string(),
                modifiers: vec!["Control".to_string()],
                check_mode: "single".to_string(),
                check_mode_param: 300,
                trigger_threshold: 0.3,
                continually_trigger: false,
                interval: 300,
                initial_interval: 300,
                min_interval: 100,
                acceleration: 0.8,
            },
        ],
        preset_config: PresetConfigData {
            deadzone: 10,
            deadzone_left: 10,
            use_stick_as_mouse: true,
            stick_as_mouse_simulation: Some("right".to_string()),
            move_speed: 25,
            stick_rotate_trigger_threshold: 15,
            sub_preset_name: None,
            sub_preset_switch_button: None,
            sub_preset_switch_mode: None,
        },
    }
}