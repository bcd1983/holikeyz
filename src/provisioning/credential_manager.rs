use anyhow::{Result, Context};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{info, debug, warn};

const SERVICE_NAME: &str = "holikeyz-ring-light";
const WIFI_CREDS_KEY: &str = "wifi-credentials";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedNetwork {
    pub ssid: String,
    pub password: Option<String>,
    pub security_type: String,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub auto_connect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedCredentials {
    pub networks: HashMap<String, SavedNetwork>,
}

impl Default for SavedCredentials {
    fn default() -> Self {
        Self {
            networks: HashMap::new(),
        }
    }
}

pub struct CredentialManager {
    entry: Entry,
}

impl CredentialManager {
    pub fn new() -> Result<Self> {
        let entry = Entry::new(SERVICE_NAME, WIFI_CREDS_KEY)?;
        Ok(Self { entry })
    }

    pub fn load_credentials(&self) -> Result<SavedCredentials> {
        match self.entry.get_password() {
            Ok(json_str) => {
                let creds: SavedCredentials = serde_json::from_str(&json_str)
                    .context("Failed to parse saved credentials")?;
                Ok(creds)
            }
            Err(_) => {
                debug!("No saved credentials found, returning empty");
                Ok(SavedCredentials::default())
            }
        }
    }

    pub fn save_credentials(&self, creds: &SavedCredentials) -> Result<()> {
        let json_str = serde_json::to_string(creds)?;
        self.entry.set_password(&json_str)
            .context("Failed to save credentials to keyring")?;
        info!("Credentials saved to system keyring");
        Ok(())
    }

    pub fn add_network(&self, network: SavedNetwork) -> Result<()> {
        let mut creds = self.load_credentials()?;
        creds.networks.insert(network.ssid.clone(), network);
        self.save_credentials(&creds)?;
        Ok(())
    }

    pub fn get_network(&self, ssid: &str) -> Result<Option<SavedNetwork>> {
        let creds = self.load_credentials()?;
        Ok(creds.networks.get(ssid).cloned())
    }

    pub fn remove_network(&self, ssid: &str) -> Result<()> {
        let mut creds = self.load_credentials()?;
        creds.networks.remove(ssid);
        self.save_credentials(&creds)?;
        Ok(())
    }

    pub fn list_networks(&self) -> Result<Vec<String>> {
        let creds = self.load_credentials()?;
        Ok(creds.networks.keys().cloned().collect())
    }

    pub fn clear_all(&self) -> Result<()> {
        self.entry.delete_password()
            .context("Failed to clear credentials")?;
        info!("All credentials cleared from keyring");
        Ok(())
    }
}

// Helper to get WiFi credentials from NetworkManager
pub async fn get_system_wifi_networks() -> Result<Vec<SavedNetwork>> {
    use std::process::Command;
    
    let output = Command::new("nmcli")
        .args(&["-t", "-f", "NAME,TYPE", "connection", "show"])
        .output()?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();
    
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 && parts[1] == "802-11-wireless" {
            let name = parts[0];
            
            // Get connection details
            let detail_output = Command::new("nmcli")
                .args(&["-t", "-s", "-f", "802-11-wireless.ssid,802-11-wireless-security.key-mgmt", 
                       "connection", "show", name])
                .output()?;
            
            let detail_str = String::from_utf8_lossy(&detail_output.stdout);
            let mut ssid = String::new();
            let mut security = String::new();
            
            for line in detail_str.lines() {
                if let Some(value) = line.strip_prefix("802-11-wireless.ssid:") {
                    ssid = value.to_string();
                } else if let Some(value) = line.strip_prefix("802-11-wireless-security.key-mgmt:") {
                    security = value.to_string();
                }
            }
            
            if !ssid.is_empty() {
                networks.push(SavedNetwork {
                    ssid: ssid.clone(),
                    password: None, // NetworkManager doesn't expose passwords
                    security_type: security,
                    last_used: None,
                    auto_connect: true,
                });
            }
        }
    }
    
    Ok(networks)
}

// Interactive credential helper
pub async fn prompt_for_credentials(ssid: &str) -> Result<crate::provisioning::WiFiCredentials> {
    use dialoguer::{Password, Select};
    use crate::provisioning::SecurityType;
    
    println!("\n📡 WiFi Network Configuration");
    println!("Network: {}", ssid);
    println!("{}", "─".repeat(40));
    
    let security_options = vec![
        "WPA/WPA2 (Most common)",
        "WPA3 (Newest)",
        "Open (No password)",
        "WEP (Legacy)",
    ];
    
    let security_idx = Select::new()
        .with_prompt("Select security type")
        .items(&security_options)
        .default(0)  // WPA/WPA2 is most common
        .interact()?;
    
    let security_type = match security_idx {
        0 => SecurityType::WPA2,
        1 => SecurityType::WPA3,
        2 => SecurityType::Open,
        3 => SecurityType::WEP,
        _ => SecurityType::WPA2,
    };
    
    let password = if security_idx != 2 {  // Not Open network
        Some(Password::new()
            .with_prompt("Enter WiFi password")
            .interact()?)
    } else {
        None
    };
    
    Ok(crate::provisioning::WiFiCredentials {
        ssid: ssid.to_string(),
        password,
        security_type,
        hidden: false,
    })
}

// Try to get credentials from various sources
pub async fn get_wifi_credentials(ssid: &str) -> Result<crate::provisioning::WiFiCredentials> {
    let manager = CredentialManager::new()?;
    
    // First check saved credentials
    if let Some(saved) = manager.get_network(ssid)? {
        info!("Using saved credentials for {}", ssid);
        return Ok(crate::provisioning::WiFiCredentials {
            ssid: saved.ssid,
            password: saved.password,
            security_type: match saved.security_type.as_str() {
                "Open" => crate::provisioning::SecurityType::Open,
                "WEP" => crate::provisioning::SecurityType::WEP,
                "WPA" => crate::provisioning::SecurityType::WPA,
                "WPA2" => crate::provisioning::SecurityType::WPA2,
                "WPA3" => crate::provisioning::SecurityType::WPA3,
                _ => crate::provisioning::SecurityType::WPA2,
            },
            hidden: false,
        });
    }
    
    // Check if it's a known system network
    let system_networks = get_system_wifi_networks().await?;
    if let Some(network) = system_networks.iter().find(|n| n.ssid == ssid) {
        info!("Found {} in system networks, but password not available", ssid);
        // We have the network but not the password, prompt for it
        return prompt_for_credentials(ssid).await;
    }
    
    // Not found anywhere, prompt for credentials
    prompt_for_credentials(ssid).await
}