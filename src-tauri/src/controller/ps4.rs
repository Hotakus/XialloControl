use std::sync::RwLock;

use ds4_rs;
use dualsense_rs::{DualSense};
use hidapi::{HidApi};
use once_cell::sync::Lazy;
use crate::controller::controller::DeviceInfo;

static DUALSENSE_INSTANCE: Lazy<RwLock<Option<DualSense>>> = Lazy::new(|| RwLock::new(
    None
));

pub fn poll_ps4_controller(device: &DeviceInfo) {
    // let vid = &device.vendor_id;
    // let pid = device.product_id.as_deref().unwrap();
    // let serial = device.serial_number.as_deref().unwrap();
    let path = device.device_path.as_deref().unwrap();
    let ds = DUALSENSE_INSTANCE.write().unwrap();
    if ds.is_none() {
        let mut controller = DualSense::new_path(path);
        controller.on_dpad_changed(Box::new(|dpad| {
            println!("dpad: {:#?}", dpad);
        }));
        let handle = controller.run();
        *DUALSENSE_INSTANCE.write().unwrap() = Some(controller);
        // handle.join().ok();
    } else {
        // 已有实例，进行状态更新等操作
        // let controller = ds.as_ref().unwrap();
        // controller.update_state();
    }
}


pub fn initialize() {
    println!("initialize ps4 controller");

    // let mut controller = DualSense::new_serial(vendor_id, product_id, serial);



    // controller.on_dpad_changed(Box::new(|dpad| {
    //     println!("dpad: {:#?}", dpad);
    // }));
    // let handle = controller.run();
    // handle.join().ok();
}
