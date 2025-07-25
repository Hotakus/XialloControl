use hidapi::HidApi;
use serde::{Deserialize, Serialize};
use std::fs;
use std::iter::Iterator;
use std::path::Path;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize)]
pub struct SupportedDevice {
    pub name: String,
    pub vendor_id: String,
    pub product_id: Option<String>, // 可选字段
}

pub struct SupportedConnectedDevice {
    pub name: String,
    pub vendor_id: String,
    pub product_id: String,
    pub device_path: String, // 唯一标识，可用来打开设备
}

fn default_devices() -> Vec<SupportedDevice> {
    vec![
        SupportedDevice {
            name: "Any Xbox Controller".into(),
            vendor_id: "045e".into(),
            product_id: None,
        },
        SupportedDevice {
            name: "DualShock 4 (PS4)".into(),
            vendor_id: "054c".into(),
            product_id: None,
        },
        SupportedDevice {
            name: "DualSense (PS5)".into(),
            vendor_id: "054c".into(),
            // product_id: Some("0ce6".into()),
            product_id: None,
        },
    ]
}

pub fn load_or_create_config(path: &str) -> Vec<SupportedDevice> {
    if Path::new(path).exists() {
        let data = fs::read_to_string(path).expect("Failed to read config");
        serde_json::from_str(&data).expect("Failed to parse JSON")
    } else {
        println!("🛠️  Config not found. Generating default...");

        let default = default_devices();
        let json =
            serde_json::to_string_pretty(&default).expect("Failed to serialize default config");
        fs::write(path, json).expect("Failed to write default config");

        default
    }
}

pub fn list_supported_connected_devices(
    config: &[SupportedDevice],
) -> Vec<SupportedConnectedDevice> {
    let api = HidApi::new().expect("Failed to init hidapi");
    let mut supported_devices = Vec::new();

    for device in api.device_list() {
        let vid = format!("{:04x}", device.vendor_id());
        let pid = format!("{:04x}", device.product_id());

        let matched = config.iter().find(|d| {
            d.vendor_id == vid
                && match &d.product_id {
                    Some(pid_cfg) => pid_cfg == &pid,
                    None => true,
                }
        });

        if let Some(supported) = matched {
            // device.path() 是唯一字符串路径，用于打开设备
            let device_info = SupportedConnectedDevice {
                name: device
                    .product_string()
                    .unwrap_or_default().to_string(),
                vendor_id: vid.clone(),
                product_id: pid.clone(),
                device_path: device.path().to_string_lossy().to_string(),
            };
            supported_devices.push(device_info);
        }
    }
    supported_devices
}
