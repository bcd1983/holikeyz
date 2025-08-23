use anyhow::{Result, Context};
use log::{info, debug, error, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;
use tokio::time::sleep;
use dialoguer;

use crate::provisioning::{
    WiFiNetwork, WiFiCredentials, SecurityType,
    elgato::{ElgatoProvisioner, ElgatoDeviceInfo, ElgatoLightState, ElgatoLight},
    credential_manager::{CredentialManager, SavedNetwork},
    wifi_manager::WiFiManager,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub ssid: String,
    pub bssid: Option<String>,
    pub signal_strength: i32,
    pub device_type: DeviceType,
    pub setup_mode: bool,
    pub mac_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    ElgatoRingLight,
    ElgatoKeyLight,
    ElgatoKeyLightAir,
    Unknown,
}

impl DeviceType {
    pub fn from_ssid(ssid: &str) -> Self {
        if ssid.starts_with("Elgato Ring Light") {
            DeviceType::ElgatoRingLight
        } else if ssid.starts_with("Elgato Key Light Air") {
            DeviceType::ElgatoKeyLightAir
        } else if ssid.starts_with("Elgato Key Light") {
            DeviceType::ElgatoKeyLight
        } else {
            DeviceType::Unknown
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedDevice {
    pub device_info: ElgatoDeviceInfo,
    pub ip_address: String,
    pub port: u16,
    pub connection_time: chrono::DateTime<chrono::Utc>,
    pub device_type: DeviceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightCommand {
    pub command_type: CommandType,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandType {
    TurnOn,
    TurnOff,
    SetBrightness,
    SetTemperature,
    SetScene,
    Toggle,
    FadeIn,
    FadeOut,
    Pulse,
    Rainbow,
}

pub struct EnhancedProvisioningManager {
    wifi_manager: Arc<WiFiManager>,
    credential_manager: Arc<CredentialManager>,
    discovered_devices: Arc<RwLock<Vec<DiscoveredDevice>>>,
    connected_devices: Arc<RwLock<HashMap<String, ConnectedDevice>>>,
    active_provisioner: Arc<RwLock<Option<ElgatoProvisioner>>>,
    original_network: Arc<RwLock<Option<String>>>,
    device_ip: Arc<RwLock<String>>,
}

impl EnhancedProvisioningManager {
    pub fn new(interface: Option<String>) -> Result<Self> {
        let interface = interface.unwrap_or_else(|| "wlan0".to_string());
        
        Ok(Self {
            wifi_manager: Arc::new(WiFiManager::new(interface)),
            credential_manager: Arc::new(CredentialManager::new()?),
            discovered_devices: Arc::new(RwLock::new(Vec::new())),
            connected_devices: Arc::new(RwLock::new(HashMap::new())),
            active_provisioner: Arc::new(RwLock::new(None)),
            original_network: Arc::new(RwLock::new(None)),
            device_ip: Arc::new(RwLock::new("192.168.62.1".to_string())),
        })
    }

    pub async fn scan_for_devices(&self, include_configured: bool) -> Result<Vec<DiscoveredDevice>> {
        info!("Scanning for Elgato devices...");
        
        let networks = self.wifi_manager.scan_networks().await?;
        let mut devices = Vec::new();
        
        for network in networks {
            // Check if it's an Elgato device
            if network.ssid.starts_with("Elgato") {
                let device_type = DeviceType::from_ssid(&network.ssid);
                let setup_mode = network.security_type == SecurityType::Open;
                
                devices.push(DiscoveredDevice {
                    ssid: network.ssid,
                    bssid: network.bssid,
                    signal_strength: network.signal_strength,
                    device_type,
                    setup_mode,
                    mac_address: None,
                });
            }
        }
        
        // Also discover devices already on the network if requested
        if include_configured {
            if let Ok(configured) = crate::discovery::discover_lights(Duration::from_secs(3)).await {
                for light in configured {
                    // These are already configured devices on the network
                    debug!("Found configured device: {} at {}", light.name, light.ip);
                }
            }
        }
        
        // Sort by signal strength
        devices.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));
        
        // Update our cache
        let mut discovered = self.discovered_devices.write().await;
        *discovered = devices.clone();
        
        info!("Found {} Elgato device(s)", devices.len());
        Ok(devices)
    }

    pub async fn connect_to_device(&self, device: &DiscoveredDevice) -> Result<ConnectedDevice> {
        info!("Connecting to device: {}", device.ssid);
        
        // Save current network
        let current = self.wifi_manager.get_current_connection().await?;
        let mut orig_net = self.original_network.write().await;
        *orig_net = current;
        
        // Create provisioner and connect
        let mut provisioner = ElgatoProvisioner::new();
        provisioner.connect_to_elgato(&device.ssid).await?;
        
        // Get device info
        let device_info = provisioner.get_device_info().await?;
        
        let ip = "192.168.62.1".to_string(); // Default Elgato AP IP
        let connected = ConnectedDevice {
            device_info: device_info.clone(),
            ip_address: ip.clone(),
            port: 9123,
            connection_time: chrono::Utc::now(),
            device_type: device.device_type.clone(),
        };
        
        // Store the device IP
        let mut device_ip = self.device_ip.write().await;
        *device_ip = ip;
        
        // Store the provisioner for later use
        let mut active = self.active_provisioner.write().await;
        *active = Some(provisioner);
        
        // Store connected device info
        let mut devices = self.connected_devices.write().await;
        devices.insert(device.ssid.clone(), connected.clone());
        
        Ok(connected)
    }

    pub async fn send_command(&self, device_id: &str, command: LightCommand) -> Result<()> {
        let provisioner = self.active_provisioner.read().await;
        let provisioner = provisioner.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No active provisioner"))?;
        
        // Get current state
        let mut state = provisioner.get_light_state().await?;
        
        // Apply command
        match command.command_type {
            CommandType::TurnOn => {
                for light in &mut state.lights {
                    light.on = 1;
                }
            }
            CommandType::TurnOff => {
                for light in &mut state.lights {
                    light.on = 0;
                }
            }
            CommandType::SetBrightness => {
                if let Some(brightness) = command.parameters.get("brightness") {
                    if let Some(b) = brightness.as_u64() {
                        for light in &mut state.lights {
                            light.brightness = (b as u8).min(100);
                        }
                    }
                }
            }
            CommandType::SetTemperature => {
                if let Some(temp) = command.parameters.get("temperature") {
                    if let Some(t) = temp.as_u64() {
                        for light in &mut state.lights {
                            light.temperature = t as u16;
                        }
                    }
                }
            }
            CommandType::Toggle => {
                for light in &mut state.lights {
                    light.on = if light.on == 1 { 0 } else { 1 };
                }
            }
            CommandType::SetScene => {
                if let Some(scene) = command.parameters.get("scene") {
                    if let Some(scene_name) = scene.as_str() {
                        apply_scene(&mut state, scene_name);
                    }
                }
            }
            CommandType::FadeIn => {
                // Implement fade in animation
                for i in 0..=100 {
                    for light in &mut state.lights {
                        light.on = 1;
                        light.brightness = i;
                    }
                    provisioner.set_light_state(&state).await?;
                    sleep(Duration::from_millis(20)).await;
                }
            }
            CommandType::FadeOut => {
                // Implement fade out animation
                let original_brightness = state.lights[0].brightness;
                for i in (0..=original_brightness).rev() {
                    for light in &mut state.lights {
                        light.brightness = i;
                    }
                    provisioner.set_light_state(&state).await?;
                    sleep(Duration::from_millis(20)).await;
                }
                for light in &mut state.lights {
                    light.on = 0;
                }
            }
            CommandType::Pulse => {
                // Pulse effect
                let original = state.clone();
                for _ in 0..3 {
                    // Dim
                    for light in &mut state.lights {
                        light.brightness = 20;
                    }
                    provisioner.set_light_state(&state).await?;
                    sleep(Duration::from_millis(500)).await;
                    
                    // Bright
                    for light in &mut state.lights {
                        light.brightness = 100;
                    }
                    provisioner.set_light_state(&state).await?;
                    sleep(Duration::from_millis(500)).await;
                }
                provisioner.set_light_state(&original).await?;
                return Ok(());
            }
            CommandType::Rainbow => {
                // Cycle through color temperatures
                let temps = vec![143, 200, 250, 300, 344];
                for temp in temps {
                    for light in &mut state.lights {
                        light.temperature = temp;
                    }
                    provisioner.set_light_state(&state).await?;
                    sleep(Duration::from_millis(500)).await;
                }
            }
        }
        
        // Apply the state change (for non-animated commands)
        if !matches!(command.command_type, CommandType::FadeIn | CommandType::FadeOut | CommandType::Pulse | CommandType::Rainbow) {
            provisioner.set_light_state(&state).await?;
        }
        
        Ok(())
    }

    pub async fn provision_device(
        &self,
        device: &DiscoveredDevice,
        target_network: &str,
        use_saved_credentials: bool,
    ) -> Result<()> {
        info!("Provisioning {} to network: {}", device.ssid, target_network);
        
        // Connect to device if not already connected
        if !self.connected_devices.read().await.contains_key(&device.ssid) {
            self.connect_to_device(device).await?;
        }
        
        // Get credentials - try saved first, then prompt if needed
        let credentials = if use_saved_credentials {
            // Try to load from credential manager
            if let Some(saved) = self.credential_manager.get_network(target_network)? {
                info!("Using saved credentials for {}", target_network);
                WiFiCredentials {
                    ssid: saved.ssid,
                    password: saved.password,
                    security_type: parse_security_type(&saved.security_type),
                    hidden: false,
                }
            } else {
                info!("No saved credentials found for {}, prompting user", target_network);
                // Prompt for credentials if not found
                let creds = crate::provisioning::credential_manager::prompt_for_credentials(target_network).await?;
                
                // Offer to save the new credentials
                println!("Would you like to save these credentials for future use?");
                if dialoguer::Confirm::new().default(true).interact()? {
                    self.credential_manager.add_network(SavedNetwork {
                        ssid: creds.ssid.clone(),
                        password: creds.password.clone(),
                        security_type: format!("{:?}", creds.security_type),
                        last_used: Some(chrono::Utc::now()),
                        auto_connect: true,
                    })?;
                    info!("Credentials saved for future use");
                }
                creds
            }
        } else {
            // Use prompt helper
            crate::provisioning::credential_manager::prompt_for_credentials(target_network).await?
        };
        
        // Note: removed the duplicate save logic since we now save when prompting
        
        // Provision the device
        let provisioner = self.active_provisioner.read().await;
        let provisioner = provisioner.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No active provisioner"))?;
        
        // Try multiple provisioning methods
        let device_ip = self.device_ip.read().await.clone();
        
        // First try the improved direct provisioning
        info!("Attempting direct provisioning methods...");
        let direct_provisioner = crate::provisioning::elgato_fixed::ElgatoDirectProvisioner::with_ip(&device_ip);
        
        match direct_provisioner.provision_device(&credentials.ssid, credentials.password.as_deref().unwrap_or("")).await {
            Ok(_) => {
                info!("Device provisioned successfully via direct API");
                println!("✅ WiFi configuration sent successfully!");
            }
            Err(direct_err) => {
                warn!("Direct provisioning failed: {}", direct_err);
                
                // Try the original encrypted method
                info!("Trying encrypted provisioning method...");
                match provisioner.provision_with_credentials(&credentials).await {
                    Ok(_) => {
                        info!("Device provisioned successfully via encrypted API");
                        println!("✅ WiFi configuration sent successfully!");
                    }
                    Err(encrypted_err) => {
                        error!("Encrypted provisioning also failed: {}", encrypted_err);
                        
                        // Last resort - show error details
                        println!("\n❌ Provisioning failed with both methods:");
                        println!("  Direct API: {}", direct_err);
                        println!("  Encrypted API: {}", encrypted_err);
                        
                        // Check endpoints to help debug
                        println!("\n🔍 Checking device endpoints...");
                        if let Err(e) = direct_provisioner.check_endpoints().await {
                            println!("  Could not check endpoints: {}", e);
                        }
                        
                        return Err(anyhow::anyhow!("Provisioning failed. Device may have different firmware version."));
                    }
                }
            }
        }
        
        // Schedule network restoration
        let original = self.original_network.read().await.clone();
        if let Some(network) = original {
            tokio::spawn(async move {
                sleep(Duration::from_secs(5)).await;
                if let Err(e) = restore_network(&network).await {
                    error!("Failed to restore network: {}", e);
                }
            });
        }
        
        Ok(())
    }

    pub async fn get_saved_networks(&self) -> Result<Vec<String>> {
        self.credential_manager.list_networks()
    }

    pub async fn save_network_credentials(&self, credentials: WiFiCredentials) -> Result<()> {
        self.credential_manager.add_network(SavedNetwork {
            ssid: credentials.ssid.clone(),
            password: credentials.password,
            security_type: format!("{:?}", credentials.security_type),
            last_used: Some(chrono::Utc::now()),
            auto_connect: true,
        })
    }

    pub async fn restore_original_network(&self) -> Result<()> {
        if let Some(network) = self.original_network.read().await.as_ref() {
            info!("Restoring connection to: {}", network);
            restore_network(network).await?;
        }
        Ok(())
    }

    pub async fn batch_provision_devices(
        &self,
        devices: Vec<DiscoveredDevice>,
        target_network: &str,
    ) -> Result<Vec<(String, Result<()>)>> {
        let mut results = Vec::new();
        
        for device in devices {
            let result = self.provision_device(&device, target_network, true).await;
            results.push((device.ssid.clone(), result));
            
            // Wait a bit between devices
            sleep(Duration::from_secs(2)).await;
        }
        
        Ok(results)
    }

    pub async fn verify_device_on_network(&self, timeout_secs: u64) -> Result<bool> {
        info!("Waiting for device to appear on network...");
        
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        
        while start.elapsed() < timeout {
            if let Ok(lights) = crate::discovery::discover_lights(Duration::from_secs(3)).await {
                if !lights.is_empty() {
                    info!("Device found on network!");
                    return Ok(true);
                }
            }
            sleep(Duration::from_secs(3)).await;
        }
        
        Ok(false)
    }
}

fn apply_scene(state: &mut ElgatoLightState, scene_name: &str) {
    let (brightness, temperature) = match scene_name.to_lowercase().as_str() {
        "daylight" => (100, 143),     // 7000K - Cool daylight
        "reading" => (80, 200),        // 5000K - Neutral white
        "video" => (90, 215),          // 4500K - Slightly warm
        "relax" => (60, 280),          // 3500K - Warm white
        "warm" => (70, 344),           // 2900K - Very warm
        "cool" => (85, 143),           // 7000K - Cool white
        "focus" => (100, 180),         // 5500K - Bright neutral
        "evening" => (50, 320),        // 3100K - Warm dim
        _ => (75, 230),                // Default
    };
    
    for light in &mut state.lights {
        light.on = 1;
        light.brightness = brightness;
        light.temperature = temperature;
    }
}

fn parse_security_type(security_str: &str) -> SecurityType {
    match security_str {
        "Open" => SecurityType::Open,
        "WEP" => SecurityType::WEP,
        "WPA" => SecurityType::WPA,
        "WPA2" => SecurityType::WPA2,
        "WPA3" => SecurityType::WPA3,
        "Enterprise" => SecurityType::Enterprise,
        _ => SecurityType::WPA2,
    }
}

async fn restore_network(network_name: &str) -> Result<()> {
    std::process::Command::new("nmcli")
        .args(&["connection", "up", network_name])
        .output()
        .context("Failed to restore network connection")?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub device_id: String,
    pub nickname: String,
    pub default_scene: Option<String>,
    pub auto_connect: bool,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

impl EnhancedProvisioningManager {
    pub async fn create_device_profile(&self, device: &ConnectedDevice, nickname: String) -> Result<DeviceProfile> {
        let profile = DeviceProfile {
            device_id: device.device_info.serial_number.clone(),
            nickname,
            default_scene: Some("daylight".to_string()),
            auto_connect: true,
            last_seen: chrono::Utc::now(),
        };
        
        // Could save this to a profiles database/file
        Ok(profile)
    }
    
    pub async fn apply_profile(&self, profile: &DeviceProfile) -> Result<()> {
        if let Some(scene) = &profile.default_scene {
            let mut params = HashMap::new();
            params.insert("scene".to_string(), serde_json::Value::String(scene.clone()));
            
            self.send_command(&profile.device_id, LightCommand {
                command_type: CommandType::SetScene,
                parameters: params,
            }).await?;
        }
        Ok(())
    }
}