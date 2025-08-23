use anyhow::{Result, Context};
use async_trait::async_trait;
use log::{info, debug, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use tokio::time::sleep;

use crate::provisioning::{
    DeviceInfo, WiFiNetwork, WiFiCredentials, ProvisioningRequest, 
    ProvisioningResponse, ProvisioningStatus, ProvisioningService,
    soft_ap::{SoftAPManager, SoftAPConfig},
    wifi_manager::WiFiManager,
    security::SecurityManager,
};

const ELGATO_PROVISIONING_PORT: u16 = 9123;
const ELGATO_DEFAULT_IP: &str = "192.168.62.1";
const PROVISIONING_TIMEOUT: Duration = Duration::from_secs(120);
const CONNECTION_VERIFY_RETRIES: u32 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ElgatoProvisioningPayload {
    ssid: String,
    pass: String,
    security: u8,
    priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ElgatoDeviceResponse {
    product_name: String,
    hardware_board_type: u32,
    firmware_build_number: u32,
    firmware_version: String,
    serial_number: String,
    display_name: String,
}

pub struct ElgatoProvisioner {
    client: Client,
    wifi_manager: WiFiManager,
    security_manager: SecurityManager,
    soft_ap_manager: Option<SoftAPManager>,
    device_ip: Option<IpAddr>,
}

impl ElgatoProvisioner {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            wifi_manager: WiFiManager::new("wlan0".to_string()),
            security_manager: SecurityManager::new(),
            soft_ap_manager: None,
            device_ip: None,
        }
    }

    async fn connect_to_device_ap(&mut self, ssid: &str) -> Result<IpAddr> {
        info!("Connecting to device's Soft AP: {}", ssid);
        
        let credentials = WiFiCredentials {
            ssid: ssid.to_string(),
            password: None,
            security_type: crate::provisioning::SecurityType::Open,
            hidden: false,
        };
        
        self.wifi_manager.connect_to_network(&credentials).await?;
        
        sleep(Duration::from_secs(3)).await;
        
        let device_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 4, 1));
        self.device_ip = Some(device_ip);
        
        Ok(device_ip)
    }

    async fn send_provisioning_payload(&self, credentials: &WiFiCredentials) -> Result<()> {
        let device_ip = self.device_ip.context("Device IP not set")?;
        
        let security_type = match credentials.security_type {
            crate::provisioning::SecurityType::Open => 0,
            crate::provisioning::SecurityType::WEP => 1,
            crate::provisioning::SecurityType::WPA => 2,
            crate::provisioning::SecurityType::WPA2 => 3,
            crate::provisioning::SecurityType::WPA3 => 4,
            crate::provisioning::SecurityType::Enterprise => {
                return Err(anyhow::anyhow!("Enterprise security not supported"));
            }
        };
        
        let payload = ElgatoProvisioningPayload {
            ssid: credentials.ssid.clone(),
            pass: credentials.password.clone().unwrap_or_default(),
            security: security_type,
            priority: 1,
        };
        
        let url = format!("http://{}:{}/elgato/wifi/update", device_ip, ELGATO_PROVISIONING_PORT);
        
        debug!("Sending provisioning payload to {}", url);
        
        let response = self.client
            .put(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send provisioning payload")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Provisioning failed: {} - {}", status, body));
        }
        
        Ok(())
    }

    async fn verify_device_connection(&self, expected_ssid: &str) -> Result<bool> {
        info!("Verifying device connection to {}", expected_ssid);
        
        for attempt in 1..=CONNECTION_VERIFY_RETRIES {
            debug!("Connection verification attempt {}/{}", attempt, CONNECTION_VERIFY_RETRIES);
            
            sleep(Duration::from_secs(5)).await;
            
            match self.discover_device_on_network(expected_ssid).await {
                Ok(true) => {
                    info!("Device successfully connected to target network");
                    return Ok(true);
                }
                Ok(false) => {
                    debug!("Device not found on network yet");
                }
                Err(e) => {
                    warn!("Error during discovery: {}", e);
                }
            }
        }
        
        Ok(false)
    }

    async fn discover_device_on_network(&self, _ssid: &str) -> Result<bool> {
        use crate::discovery::discover_lights;
        
        let lights = discover_lights(Duration::from_secs(5)).await?;
        
        Ok(!lights.is_empty())
    }

    async fn get_elgato_device_info(&self) -> Result<ElgatoDeviceResponse> {
        let device_ip = self.device_ip.context("Device IP not set")?;
        let url = format!("http://{}:{}/elgato/accessory-info", device_ip, ELGATO_PROVISIONING_PORT);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to get device info")?;
        
        let device_info: ElgatoDeviceResponse = response
            .json()
            .await
            .context("Failed to parse device info")?;
        
        Ok(device_info)
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
        info!("Starting Elgato device provisioning for session {}", request.session_id);
        
        let mut response = ProvisioningResponse {
            session_id: request.session_id.clone(),
            status: ProvisioningStatus::Connecting,
            message: Some("Sending WiFi credentials to device".to_string()),
            device_info: None,
        };
        
        self.send_provisioning_payload(&request.wifi_credentials).await?;
        
        response.status = ProvisioningStatus::Authenticating;
        response.message = Some("Device is connecting to WiFi network".to_string());
        
        sleep(Duration::from_secs(10)).await;
        
        self.wifi_manager.disconnect_from_network().await.ok();
        self.wifi_manager.connect_to_network(&request.wifi_credentials).await?;
        
        response.status = ProvisioningStatus::Configuring;
        response.message = Some("Verifying device connection".to_string());
        
        let connected = self.verify_device_connection(&request.wifi_credentials.ssid).await?;
        
        if connected {
            response.status = ProvisioningStatus::Success;
            response.message = Some("Device successfully provisioned".to_string());
        } else {
            response.status = ProvisioningStatus::Failed;
            response.message = Some("Failed to verify device connection".to_string());
        }
        
        Ok(response)
    }

    async fn get_device_info(&self) -> Result<DeviceInfo> {
        let elgato_info = self.get_elgato_device_info().await?;
        
        Ok(DeviceInfo {
            device_id: elgato_info.serial_number.clone(),
            device_type: "Ring Light".to_string(),
            manufacturer: "Elgato".to_string(),
            model: elgato_info.product_name,
            firmware_version: elgato_info.firmware_version,
            mac_address: String::new(),
            capabilities: vec![
                "brightness".to_string(),
                "temperature".to_string(),
                "on_off".to_string(),
            ],
        })
    }

    async fn verify_connection(&self) -> Result<bool> {
        self.wifi_manager.verify_internet_connection().await
    }
}

pub struct GenericProvisioner {
    wifi_manager: WiFiManager,
    security_manager: SecurityManager,
    client: Client,
}

impl GenericProvisioner {
    pub fn new() -> Self {
        Self {
            wifi_manager: WiFiManager::new("wlan0".to_string()),
            security_manager: SecurityManager::new(),
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl ProvisioningService for GenericProvisioner {
    async fn start_soft_ap(&self, ssid: &str, password: Option<&str>) -> Result<IpAddr> {
        let config = SoftAPConfig {
            ssid: ssid.to_string(),
            password: password.map(|p| p.to_string()),
            ..Default::default()
        };
        
        let mut manager = SoftAPManager::new(config);
        manager.start().await
    }

    async fn stop_soft_ap(&self) -> Result<()> {
        Ok(())
    }

    async fn scan_wifi_networks(&self) -> Result<Vec<WiFiNetwork>> {
        self.wifi_manager.scan_networks().await
    }

    async fn provision_device(&self, request: ProvisioningRequest) -> Result<ProvisioningResponse> {
        self.wifi_manager.connect_to_network(&request.wifi_credentials).await?;
        
        let connected = self.wifi_manager.verify_internet_connection().await?;
        
        Ok(ProvisioningResponse {
            session_id: request.session_id,
            status: if connected { ProvisioningStatus::Success } else { ProvisioningStatus::Failed },
            message: Some(if connected { 
                "Successfully connected to network".to_string() 
            } else { 
                "Failed to connect to network".to_string() 
            }),
            device_info: None,
        })
    }

    async fn get_device_info(&self) -> Result<DeviceInfo> {
        let info = self.wifi_manager.get_interface_info().await?;
        
        Ok(DeviceInfo {
            device_id: info.mac_address.clone(),
            device_type: "Generic Device".to_string(),
            manufacturer: "Unknown".to_string(),
            model: "Unknown".to_string(),
            firmware_version: "Unknown".to_string(),
            mac_address: info.mac_address,
            capabilities: vec![],
        })
    }

    async fn verify_connection(&self) -> Result<bool> {
        self.wifi_manager.verify_internet_connection().await
    }
}