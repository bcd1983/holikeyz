use anyhow::{Result, Context};
use async_trait::async_trait;
use log::{info, debug, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use tokio::time::sleep;
use rand::{RngCore, rngs::OsRng};

use crate::provisioning::{
    DeviceInfo, WiFiNetwork, WiFiCredentials, ProvisioningRequest, 
    ProvisioningResponse, ProvisioningStatus, ProvisioningService,
    wifi_manager::WiFiManager,
};

const ELGATO_PORT: u16 = 9123;
const ELGATO_DEFAULT_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 62, 1);
const ELGATO_FIXED_IV: &str = "049F6F1149C6F84B1B14913C71E9CDBE";
const ELGATO_BASE_KEY: &str = "4CB4btbtB0EADDEEEB2A038A31fwfw56";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElgatoDeviceInfo {
    #[serde(rename = "productName")]
    pub product_name: String,
    #[serde(rename = "hardwareBoardType")]
    pub hardware_board_type: u32,
    #[serde(rename = "hardwareRevision")]
    pub hardware_revision: f32,
    #[serde(rename = "macAddress")]
    pub mac_address: String,
    #[serde(rename = "firmwareBuildNumber")]
    pub firmware_build_number: u32,
    #[serde(rename = "firmwareVersion")]
    pub firmware_version: String,
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub features: Vec<String>,
    #[serde(rename = "wifi-info")]
    pub wifi_info: Option<ElgatoWifiInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElgatoWifiInfo {
    pub ssid: String,
    #[serde(rename = "frequencyMHz")]
    pub frequency_mhz: u32,
    pub rssi: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElgatoLightState {
    #[serde(rename = "numberOfLights")]
    pub number_of_lights: u32,
    pub lights: Vec<ElgatoLight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElgatoLight {
    pub on: u8,
    pub brightness: u8,
    pub temperature: u16,
}

#[derive(Debug, Clone, Serialize)]
struct ElgatoWifiPayload {
    #[serde(rename = "SSID")]
    ssid: String,
    #[serde(rename = "Passphrase")]
    passphrase: String,
    #[serde(rename = "SecurityType")]
    security_type: String,
}

pub struct ElgatoProvisioner {
    client: Client,
    wifi_manager: WiFiManager,
    device_ip: IpAddr,
    original_network: Option<String>,
}

impl ElgatoProvisioner {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            wifi_manager: WiFiManager::new("wlan0".to_string()),
            device_ip: IpAddr::V4(ELGATO_DEFAULT_IP),
            original_network: None,
        }
    }

    pub async fn scan_for_elgato_networks(&self) -> Result<Vec<WiFiNetwork>> {
        info!("Scanning for Elgato Ring Light networks...");
        
        let networks = self.wifi_manager.scan_networks().await?;
        let elgato_networks: Vec<WiFiNetwork> = networks
            .into_iter()
            .filter(|n| n.ssid.starts_with("Elgato Ring Light") || n.ssid.starts_with("Elgato Key Light"))
            .collect();
        
        info!("Found {} Elgato device(s)", elgato_networks.len());
        Ok(elgato_networks)
    }

    pub async fn connect_to_elgato(&mut self, ssid: &str) -> Result<()> {
        info!("Connecting to Elgato device: {}", ssid);
        
        // Save current network for later restoration
        self.original_network = self.wifi_manager.get_current_connection().await?;
        
        // Connect to Elgato AP (typically open network)
        let credentials = WiFiCredentials {
            ssid: ssid.to_string(),
            password: None,
            security_type: crate::provisioning::SecurityType::Open,
            hidden: false,
        };
        
        self.wifi_manager.connect_to_network(&credentials).await?;
        
        // Wait for connection to establish
        sleep(Duration::from_secs(3)).await;
        
        // Try to find the device IP
        if let Ok(device_info) = self.get_device_info_at_ip(IpAddr::V4(ELGATO_DEFAULT_IP)).await {
            self.device_ip = IpAddr::V4(ELGATO_DEFAULT_IP);
            info!("Connected to Elgato device: {}", device_info.serial_number);
        } else {
            // Try alternative IPs
            for ip in &[
                Ipv4Addr::new(192, 168, 4, 1),
                Ipv4Addr::new(10, 123, 45, 1),
                Ipv4Addr::new(172, 16, 0, 1),
            ] {
                if let Ok(device_info) = self.get_device_info_at_ip(IpAddr::V4(*ip)).await {
                    self.device_ip = IpAddr::V4(*ip);
                    info!("Found Elgato device at {}: {}", ip, device_info.serial_number);
                    break;
                }
            }
        }
        
        Ok(())
    }

    async fn get_device_info_at_ip(&self, ip: IpAddr) -> Result<ElgatoDeviceInfo> {
        let url = format!("http://{}:{}/elgato/accessory-info", ip, ELGATO_PORT);
        let response = self.client.get(&url).send().await?;
        let device_info: ElgatoDeviceInfo = response.json().await?;
        Ok(device_info)
    }

    pub async fn get_device_info(&self) -> Result<ElgatoDeviceInfo> {
        self.get_device_info_at_ip(self.device_ip).await
    }

    pub async fn get_light_state(&self) -> Result<ElgatoLightState> {
        let url = format!("http://{}:{}/elgato/lights", self.device_ip, ELGATO_PORT);
        let response = self.client.get(&url).send().await?;
        let state: ElgatoLightState = response.json().await?;
        Ok(state)
    }

    pub async fn set_light_state(&self, state: &ElgatoLightState) -> Result<()> {
        let url = format!("http://{}:{}/elgato/lights", self.device_ip, ELGATO_PORT);
        self.client.put(&url).json(state).send().await?;
        Ok(())
    }

    fn generate_encryption_key(&self, device_info: &ElgatoDeviceInfo) -> String {
        let firmware_hex = format!("{:04x}", device_info.firmware_build_number);
        let hardware_hex = format!("{:04x}", device_info.hardware_board_type);
        
        // Little-endian byte swap
        let firmware_lsb = format!("{}{}", &firmware_hex[2..], &firmware_hex[..2]);
        let hardware_lsb = format!("{}{}", &hardware_hex[2..], &hardware_hex[..2]);
        
        // Replace placeholders in base key
        ELGATO_BASE_KEY
            .replace("btbt", &hardware_lsb)
            .replace("fwfw", &firmware_lsb)
    }

    fn encrypt_wifi_credentials(&self, credentials: &WiFiCredentials, key: &str) -> Result<Vec<u8>> {
        // Create JSON payload
        let security_type = match credentials.security_type {
            crate::provisioning::SecurityType::Open => "0",
            crate::provisioning::SecurityType::WEP => "1",
            crate::provisioning::SecurityType::WPA | 
            crate::provisioning::SecurityType::WPA2 | 
            crate::provisioning::SecurityType::WPA3 => "2",
            crate::provisioning::SecurityType::Enterprise => {
                return Err(anyhow::anyhow!("Enterprise security not supported"));
            }
        };
        
        let payload = ElgatoWifiPayload {
            ssid: credentials.ssid.clone(),
            passphrase: credentials.password.clone().unwrap_or_default(),
            security_type: security_type.to_string(),
        };
        
        let json_str = serde_json::to_string(&payload)?;
        let mut json_bytes = json_str.into_bytes();
        
        // Pad to multiple of 16
        let padding_len = 16 - (json_bytes.len() % 16);
        json_bytes.extend(vec![padding_len as u8; padding_len]);
        
        // Add 16-byte random prefix — use OS CSPRNG for crypto material
        let mut random_prefix = vec![0u8; 16];
        OsRng.fill_bytes(&mut random_prefix);
        
        // Combine random prefix with JSON data
        let mut data_to_encrypt = random_prefix;
        data_to_encrypt.extend(json_bytes);
        
        // Parse key and IV
        let key_bytes = hex::decode(key)?;
        let iv_bytes = hex::decode(ELGATO_FIXED_IV)?;
        
        // Ensure key is 32 bytes for AES-256
        if key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid key length: expected 32 bytes, got {}", key_bytes.len()));
        }
        
        // Ensure IV is 16 bytes
        if iv_bytes.len() != 16 {
            return Err(anyhow::anyhow!("Invalid IV length: expected 16 bytes, got {}", iv_bytes.len()));
        }
        
        // Perform AES-256-CBC encryption
        use aes::Aes256;
        use cbc::{Encryptor, cipher::{BlockEncryptMut, KeyIvInit}};
        
        type Aes256CbcEnc = Encryptor<Aes256>;
        
        let key_array: [u8; 32] = key_bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Failed to convert key to array"))?;
        let iv_array: [u8; 16] = iv_bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Failed to convert IV to array"))?;
        
        let cipher = Aes256CbcEnc::new(&key_array.into(), &iv_array.into());
        let encrypted = cipher.encrypt_padded_vec_mut::<cbc::cipher::block_padding::NoPadding>(&data_to_encrypt);
        
        debug!("Encrypted {} bytes of WiFi credentials", encrypted.len());
        Ok(encrypted)
    }

    pub async fn provision_with_credentials(&self, credentials: &WiFiCredentials) -> Result<()> {
        info!("Provisioning Elgato device with network: {}", credentials.ssid);
        
        // Get device info for encryption key
        let device_info = self.get_device_info().await?;
        let encryption_key = self.generate_encryption_key(&device_info);

        // Encrypt credentials
        let encrypted_data = self.encrypt_wifi_credentials(credentials, &encryption_key)?;
        
        // Send encrypted credentials
        let url = format!("http://{}:{}/elgato/wifi-info", self.device_ip, ELGATO_PORT);
        let response = self.client
            .put(&url)
            .header("Content-Type", "application/octet-stream")
            .body(encrypted_data)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("WiFi credentials sent successfully. Device will reboot.");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to send WiFi credentials: {}", response.status()))
        }
    }

    pub async fn restore_original_network(&self) -> Result<()> {
        if let Some(network) = &self.original_network {
            info!("Restoring connection to original network: {}", network);
            // Try to reconnect using NetworkManager's saved profile
            std::process::Command::new("nmcli")
                .args(&["connection", "up", network])
                .output()?;
        }
        Ok(())
    }

    pub async fn scan_wifi_networks(&self) -> Result<Vec<WiFiNetwork>> {
        self.wifi_manager.scan_networks().await
    }

    pub async fn wait_for_device_on_network(&self, timeout_secs: u64) -> Result<bool> {
        info!("Waiting for device to appear on network...");
        
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        
        while start.elapsed() < timeout {
            if let Ok(lights) = crate::discovery::discover_lights(Duration::from_secs(5)).await {
                if !lights.is_empty() {
                    info!("Device found on network!");
                    return Ok(true);
                }
            }
            sleep(Duration::from_secs(5)).await;
        }
        
        Ok(false)
    }
}

#[async_trait]
impl ProvisioningService for ElgatoProvisioner {
    async fn start_soft_ap(&self, _ssid: &str, _password: Option<&str>) -> Result<IpAddr> {
        Err(anyhow::anyhow!("Elgato devices create their own Soft AP"))
    }

    async fn stop_soft_ap(&self) -> Result<()> {
        Ok(())
    }

    async fn scan_wifi_networks(&self) -> Result<Vec<WiFiNetwork>> {
        self.wifi_manager.scan_networks().await
    }

    async fn provision_device(&self, request: ProvisioningRequest) -> Result<ProvisioningResponse> {
        self.provision_with_credentials(&request.wifi_credentials).await?;
        
        Ok(ProvisioningResponse {
            session_id: request.session_id,
            status: ProvisioningStatus::Success,
            message: Some("Device provisioned successfully".to_string()),
            device_info: None,
        })
    }

    async fn get_device_info(&self) -> Result<DeviceInfo> {
        let elgato_info = self.get_device_info().await?;
        
        Ok(DeviceInfo {
            device_id: elgato_info.serial_number.clone(),
            device_type: "Ring Light".to_string(),
            manufacturer: "Elgato".to_string(),
            model: elgato_info.product_name,
            firmware_version: elgato_info.firmware_version,
            mac_address: elgato_info.mac_address,
            capabilities: elgato_info.features,
        })
    }

    async fn verify_connection(&self) -> Result<bool> {
        self.wifi_manager.verify_internet_connection().await
    }
}