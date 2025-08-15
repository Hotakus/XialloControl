
// 1. check stick stilled
// 2. push stick to max range
// 3. rotate stick 360 degrees (check max range)

pub enum StickTestSteps {
    Idle,
    CenterCheck,
    MaxRange,
    Rotation,
    Complete,
}

pub struct StickCalibration {
    step: StickTestSteps,
    stick_center: (f32, f32),
    stick_range: (f32, f32),
}

impl StickCalibration {
    pub fn new() -> StickCalibration {
        StickCalibration {
            step: StickTestSteps::Idle,
            stick_center: (0.0, 0.0),
            stick_range: (0.0, 0.0),
        }
    }

    pub fn update_step_to(&mut self, step: StickTestSteps) {
        self.step = step;
    }

    
}
