use once_cell::sync::Lazy;
use crate::controller;
use crate::controller::{ControllerType, DeviceInfo};

static SUPPORTED_DEVICES_DEFAULT: Lazy<Vec<DeviceInfo>> = Lazy::new(|| {
    vec![
        DeviceInfo {
            name: "Any Xbox Controller".into(),
            vendor_id: "045e".into(),
            product_id: None,
            device_path: None,
            controller_type: ControllerType::Xbox,
        },
        DeviceInfo {
            name: "Any Xbox Controller".into(),
            vendor_id: "045e".into(),
            product_id: None,
            device_path: None,
            controller_type: ControllerType::Xbox,
        },
    ]
});

pub fn check_type(device_info: &DeviceInfo) {

}

pub fn initialize() {

}