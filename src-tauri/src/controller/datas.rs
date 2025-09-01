use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum JoystickRotation {
    #[default]
    None,
    Clockwise,
    CounterClockwise,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ControllerStick {
    pub x: f32,
    pub y: f32,
    pub is_pressed: bool, // true if the stick is pressed, false otherwise
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ControllerTrigger {
    pub value: f32,
    pub has_pressure: bool, // true if the trigger has pressure, false otherwise
    pub is_pressed: bool,   // true if pressed, false if released
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ControllerLimits {
    pub sticks_value_min: f32,
    pub sticks_value_max: f32,
    pub triggers_value_min: f32,
    pub triggers_value_max: f32,
}

impl ControllerLimits {
    pub(crate) fn default() -> Self {
        ControllerLimits {
            sticks_value_min: -32768.0,
            sticks_value_max: 32767.0,
            triggers_value_min: 0.0,
            triggers_value_max: 255.0,
        }
    }

    pub fn set_limits(
        &mut self,
        sticks_value_min: f32,
        sticks_value_max: f32,
        triggers_value_min: f32,
        triggers_value_max: f32,
    ) {
        self.sticks_value_min = sticks_value_min;
        self.sticks_value_max = sticks_value_max;
        self.triggers_value_min = triggers_value_min;
        self.triggers_value_max = triggers_value_max;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ControllerButtons {
    // Face buttons
    South = 0,
    East,
    West,
    North,

    // Shoulder buttons
    LB,
    RB,

    // Thumb buttons
    LStick,
    RStick,

    // Back button
    Back,

    // Start button
    Start,

    // Guide button
    Guide,

    // D-pad
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ControllerDatas {
    pub buttons: u32, // bitfield of buttons

    pub left_stick: ControllerStick,
    pub right_stick: ControllerStick,

    pub left_trigger: ControllerTrigger,
    pub right_trigger: ControllerTrigger,

    // 新增：摇杆旋转状态
    pub left_stick_rotation: JoystickRotation,
    pub right_stick_rotation: JoystickRotation,

    pub left_stick_center: (f32, f32),
    pub right_stick_center: (f32, f32),

    pub limits: ControllerLimits,
}

impl ControllerDatas {
    pub fn new() -> ControllerDatas {
        ControllerDatas {
            buttons: 0,
            left_stick: ControllerStick {
                x: 0.0,
                y: 0.0,
                is_pressed: false,
            },
            right_stick: ControllerStick {
                x: 0.0,
                y: 0.0,
                is_pressed: false,
            },
            left_trigger: ControllerTrigger {
                value: 0.0,
                has_pressure: false,
                is_pressed: false,
            },
            right_trigger: ControllerTrigger {
                value: 0.0,
                has_pressure: false,
                is_pressed: false,
            },
            left_stick_rotation: JoystickRotation::None,
            right_stick_rotation: JoystickRotation::None,
            left_stick_center: (0.0, 0.0),
            right_stick_center: (0.0, 0.0),
            limits: ControllerLimits {
                sticks_value_min: -1.0,
                sticks_value_max: 1.0,
                triggers_value_min: 0.0,
                triggers_value_max: 255.0,
            },
        }
    }

    fn set_button_bit(&mut self, bit: u32, value: bool) {
        if value {
            self.buttons |= 1 << bit;
        } else {
            self.buttons &= !(1 << bit);
        }
    }

    pub fn set_button(&mut self, button: ControllerButtons, is_pressed: bool) {
        self.set_button_bit(button as u32, is_pressed);
    }

    fn get_button_bit(&self, bit: u32) -> bool {
        (self.buttons & (1 << bit)) != 0
    }

    pub fn get_button(&self, button: ControllerButtons) -> bool {
        self.get_button_bit(button as u32)
    }

    pub fn button_is_pressed(&self, button: ControllerButtons) -> bool {
        self.get_button(button)
    }

    pub fn as_compact(self) -> CompactControllerDatas {
        let mut d = CompactControllerDatas::new();
        d.buttons = self.buttons;
        d.pressure.left_stick_x = self.left_stick.x;
        d.pressure.left_stick_y = self.left_stick.y;
        d.pressure.right_stick_x = self.right_stick.x;
        d.pressure.right_stick_y = self.right_stick.y;
        d.pressure.left_trigger = self.left_trigger.value;
        d.pressure.right_trigger = self.right_trigger.value;
        d
    }

    pub fn as_compact_pressure(self) -> CompactPressureDatas {
        CompactPressureDatas {
            left_stick_x: self.left_stick.x,
            left_stick_y: self.left_stick.y,
            right_stick_x: self.right_stick.x,
            right_stick_y: self.right_stick.y,
            left_trigger: self.left_trigger.value,
            right_trigger: self.right_trigger.value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CompactPressureDatas {
    pub left_stick_x: f32,
    pub left_stick_y: f32,

    pub right_stick_x: f32,
    pub right_stick_y: f32,

    pub left_trigger: f32,
    pub right_trigger: f32,
}

impl CompactPressureDatas {
    pub fn new() -> CompactPressureDatas {
        Self {
            left_stick_x: 0.0,
            left_stick_y: 0.0,
            right_stick_x: 0.0,
            right_stick_y: 0.0,
            left_trigger: 0.0,
            right_trigger: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CompactControllerDatas {
    pub buttons: u32,
    pub pressure: CompactPressureDatas
}

impl CompactControllerDatas {
    pub fn new() -> Self {
        CompactControllerDatas {
            buttons: 0,
            pressure: CompactPressureDatas::new()
        }
    }
}

pub fn initialize() {
    log::debug!("初始化控制器数据");
}
