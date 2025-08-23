use anyhow::{Result, Context};
use log::{info, debug, warn};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use reqwest::Client;
use std::time::Duration;

// Elgato protocol constants
const ELGATO_PORT: u16 = 9123;
const ELGATO_DEFAULT_IP: &str = "192.168.62.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElgatoWifiConfig {
    #[serde(rename = "ssid")]
    pub ssid: String,
    #[serde(rename = "passphrase")]
    pub passphrase: String,
    #[serde(rename = "priority")]
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElgatoWifiConfigWrapper {
    #[serde(rename = "numberOfWifiConfigs")]
    pub number_of_wifi_configs: u32,
    #[serde(rename = "wifiConfigs")]
    pub wifi_configs: Vec<ElgatoWifiConfig>,
}

pub struct ElgatoDirectProvisioner {
    client: Client,
    device_ip: String,
}

impl ElgatoDirectProvisioner {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            device_ip: ELGATO_DEFAULT_IP.to_string(),
        }
    }

    pub fn with_ip(ip: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            device_ip: ip.to_string(),
        }
    }

    /// Get device information
    pub async fn get_device_info(&self) -> Result<serde_json::Value> {
        let url = format!("http://{}:{}/elgato/accessory-info", self.device_ip, ELGATO_PORT);
        debug!("Getting device info from: {}", url);
        
        let response = self.client.get(&url).send().await?;
        let info = response.json().await?;
        Ok(info)
    }

    /// Get current WiFi settings
    pub async fn get_wifi_info(&self) -> Result<serde_json::Value> {
        let url = format!("http://{}:{}/elgato/wifi-info", self.device_ip, ELGATO_PORT);
        debug!("Getting WiFi info from: {}", url);
        
        let response = self.client.get(&url).send().await?;
        let info = response.json().await?;
        Ok(info)
    }

    /// Set WiFi configuration using the simpler JSON approach
    pub async fn set_wifi_simple(&self, ssid: &str, password: &str) -> Result<()> {
        let url = format!("http://{}:{}/elgato/wifi-simple", self.device_ip, ELGATO_PORT);
        
        let config = serde_json::json!({
            "ssid": ssid,
            "passphrase": password
        });
        
        debug!("Sending simple WiFi config to: {}", url);
        debug!("Config: {:?}", config);
        
        let response = self.client
            .put(&url)
            .json(&config)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("WiFi configuration sent successfully (simple method)");
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Failed to set WiFi config: {} - {}", status, text))
        }
    }

    /// Set WiFi configuration using the newer JSON endpoint
    pub async fn set_wifi_json(&self, ssid: &str, password: &str) -> Result<()> {
        let url = format!("http://{}:{}/elgato/wifi-info", self.device_ip, ELGATO_PORT);
        
        // Try the wrapper format that some firmware versions expect
        let config = ElgatoWifiConfigWrapper {
            number_of_wifi_configs: 1,
            wifi_configs: vec![
                ElgatoWifiConfig {
                    ssid: ssid.to_string(),
                    passphrase: password.to_string(),
                    priority: 1,
                }
            ],
        };
        
        debug!("Sending WiFi config to: {}", url);
        debug!("Config: {:?}", serde_json::to_string(&config)?);
        
        let response = self.client
            .put(&url)
            .header("Content-Type", "application/json")
            .json(&config)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("WiFi configuration sent successfully (JSON method)");
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Failed to set WiFi config: {} - {}", status, text))
        }
    }

    /// Try multiple provisioning methods
    pub async fn provision_device(&self, ssid: &str, password: &str) -> Result<()> {
        info!("Attempting to provision device for network: {}", ssid);
        
        // First, verify we can reach the device
        match self.get_device_info().await {
            Ok(info) => {
                debug!("Device info: {}", serde_json::to_string_pretty(&info)?);
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Cannot reach device at {}: {}", self.device_ip, e));
            }
        }
        
        // Try different provisioning methods
        // Method 1: Simple JSON endpoint (if it exists)
        debug!("Trying simple JSON method...");
        match self.set_wifi_simple(ssid, password).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                debug!("Simple method failed: {}", e);
            }
        }
        
        // Method 2: Standard JSON endpoint with wrapper
        debug!("Trying standard JSON method...");
        match self.set_wifi_json(ssid, password).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                debug!("Standard JSON method failed: {}", e);
            }
        }
        
        // Method 3: Try without wrapper
        debug!("Trying direct JSON method...");
        let url = format!("http://{}:{}/elgato/wifi-info", self.device_ip, ELGATO_PORT);
        let config = serde_json::json!({
            "ssid": ssid,
            "passphrase": password,
            "priority": 1
        });
        
        let response = self.client
            .put(&url)
            .json(&config)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("WiFi configuration sent successfully (direct JSON)");
            return Ok(());
        }
        
        // Method 4: Try form-encoded
        debug!("Trying form-encoded method...");
        let params = [
            ("ssid", ssid),
            ("passphrase", password),
        ];
        
        let response = self.client
            .put(&url)
            .form(&params)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("WiFi configuration sent successfully (form-encoded)");
            return Ok(());
        }
        
        Err(anyhow::anyhow!("All provisioning methods failed. The device may require encrypted credentials."))
    }

    /// Restart the device to apply settings
    pub async fn restart_device(&self) -> Result<()> {
        let url = format!("http://{}:{}/elgato/restart", self.device_ip, ELGATO_PORT);
        debug!("Restarting device...");
        
        let response = self.client.post(&url).send().await?;
        
        if response.status().is_success() {
            info!("Device restart initiated");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to restart device"))
        }
    }

    /// Check available endpoints (for debugging)
    pub async fn check_endpoints(&self) -> Result<()> {
        let endpoints = vec![
            "/elgato/accessory-info",
            "/elgato/wifi-info",
            "/elgato/wifi-simple",
            "/elgato/lights",
            "/elgato/restart",
            "/elgato/identify",
        ];
        
        println!("Checking device endpoints at {}:", self.device_ip);
        for endpoint in endpoints {
            let url = format!("http://{}:{}{}", self.device_ip, ELGATO_PORT, endpoint);
            match self.client.get(&url).send().await {
                Ok(response) => {
                    println!("  {} - {} ✓", endpoint, response.status());
                }
                Err(_) => {
                    println!("  {} - unreachable ✗", endpoint);
                }
            }
        }
        
        Ok(())
    }
}

/// Alternative provisioning using different approaches
pub async fn provision_with_retry(ip: &str, ssid: &str, password: &str) -> Result<()> {
    let provisioner = ElgatoDirectProvisioner::with_ip(ip);
    
    // First check what endpoints are available
    provisioner.check_endpoints().await?;
    
    // Try provisioning
    provisioner.provision_device(ssid, password).await?;
    
    // Optionally restart
    if let Err(e) = provisioner.restart_device().await {
        warn!("Could not restart device (may restart automatically): {}", e);
    }
    
    Ok(())
}