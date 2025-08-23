use thiserror::Error;

#[derive(Error, Debug)]
pub enum HolikeyzError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    
    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Device not found at {0}")]
    DeviceNotFound(String),
    
    #[error("Invalid temperature value: {0} (must be between 143-344)")]
    InvalidTemperature(u16),
    
    #[error("Invalid brightness value: {0} (must be between 0-100)")]
    InvalidBrightness(u8),
    
    #[error("mDNS discovery failed: {0}")]
    DiscoveryError(String),
    
    #[error("D-Bus error: {0}")]
    DBusError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, HolikeyzError>;