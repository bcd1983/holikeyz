use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightResponse {
    pub number_of_lights: u8,
    pub lights: Vec<LightState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightState {
    pub on: u8,
    pub brightness: u8,
    pub temperature: u16,
}

impl LightState {
    pub fn new() -> Self {
        Self {
            on: 0,
            brightness: 50,
            temperature: 213,
        }
    }
    
    pub fn is_on(&self) -> bool {
        self.on == 1
    }
    
    pub fn set_on(&mut self, on: bool) {
        self.on = if on { 1 } else { 0 };
    }
    
    pub fn kelvin_to_api(kelvin: u32) -> u16 {
        match kelvin {
            k if k >= 7000 => 143,
            k if k <= 2900 => 344,
            k => {
                let normalized = ((7000.0 - k as f64) / (7000.0 - 2900.0)) as f64;
                let api_value = 143.0 + (normalized * (344.0 - 143.0));
                api_value.round() as u16
            }
        }
    }
    
    pub fn api_to_kelvin(api_value: u16) -> u32 {
        match api_value {
            v if v <= 143 => 7000,
            v if v >= 344 => 2900,
            v => {
                let normalized = ((v - 143) as f64) / ((344 - 143) as f64);
                let kelvin = 7000.0 - (normalized * (7000.0 - 2900.0));
                kelvin.round() as u32
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessoryInfo {
    pub product_name: String,
    pub hardware_board_type: u32,
    pub firmware_build_number: u32,
    pub firmware_version: String,
    pub serial_number: String,
    pub display_name: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub power_on_behavior: u8,
    pub power_on_brightness: u8,
    pub power_on_temperature: u16,
    pub switch_on_duration_ms: u32,
    pub switch_off_duration_ms: u32,
    pub color_change_duration_ms: u32,
}

#[derive(Debug, Clone)]
pub struct LightInfo {
    pub ip: String,
    pub port: u16,
    pub name: String,
    pub state: Option<LightState>,
    pub accessory_info: Option<AccessoryInfo>,
}