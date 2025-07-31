use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerButtons {
    // Face buttons
    A = 0,
    B,
    X,
    Y,

    // Shoulder buttons
    LB,
    RB,

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

    pub left_stick_center: (f32, f32),
    pub right_stick_center: (f32, f32),
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
            left_stick_center: (0.0, 0.0),
            right_stick_center: (0.0, 0.0),
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
        let button_tmp = self.buttons;
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

}

pub fn initialize() {
    println!("Controller datas initialized");
}
