use anyhow::{Result, Context};
use std::process::Command;
use log::{info, debug};
use crate::provisioning::{WiFiNetwork, WiFiCredentials, SecurityType};

pub struct WiFiManager {
    interface: String,
}

impl WiFiManager {
    pub fn new(interface: String) -> Self {
        Self { interface }
    }

    pub async fn scan_networks(&self) -> Result<Vec<WiFiNetwork>> {
        debug!("Scanning for WiFi networks on interface {}", self.interface);
        
        let output = Command::new("nmcli")
            .args(&["-t", "-f", "SSID,BSSID,MODE,CHAN,FREQ,RATE,SIGNAL,SECURITY", "dev", "wifi", "list"])
            .output()
            .context("Failed to scan WiFi networks")?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut networks = Vec::new();
        
        for line in output_str.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 8 && !parts[0].is_empty() {
                let network = WiFiNetwork {
                    ssid: parts[0].to_string(),
                    bssid: Some(parts[1].to_string()),
                    signal_strength: parts[6].parse().unwrap_or(0),
                    security_type: parse_security_type(parts[7]),
                    frequency: parts[4].parse().unwrap_or(0),
                    channel: parts[3].parse().unwrap_or(0),
                };
                networks.push(network);
            }
        }
        
        networks.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));
        Ok(networks)
    }

    pub async fn connect_to_network(&self, credentials: &WiFiCredentials) -> Result<()> {
        info!("Connecting to network: {}", credentials.ssid);
        
        let connection_name = format!("provisioning-{}", credentials.ssid);
        
        self.remove_connection(&connection_name).await.ok();
        
        let mut args = vec![
            "connection", "add",
            "type", "wifi",
            "con-name", &connection_name,
            "ifname", &self.interface,
            "ssid", &credentials.ssid,
        ];
        
        let security_args = match credentials.security_type {
            SecurityType::Open => vec![],
            SecurityType::WPA | SecurityType::WPA2 | SecurityType::WPA3 => {
                if let Some(password) = &credentials.password {
                    vec!["wifi-sec.key-mgmt", "wpa-psk", "wifi-sec.psk", password]
                } else {
                    return Err(anyhow::anyhow!("Password required for secured network"));
                }
            }
            SecurityType::WEP => {
                if let Some(password) = &credentials.password {
                    vec!["wifi-sec.key-mgmt", "none", "wifi-sec.wep-key0", password]
                } else {
                    return Err(anyhow::anyhow!("Password required for WEP network"));
                }
            }
            SecurityType::Enterprise => {
                return Err(anyhow::anyhow!("Enterprise networks not yet supported"));
            }
        };
        
        for arg in &security_args {
            args.push(arg);
        }
        
        let output = Command::new("nmcli")
            .args(&args)
            .output()
            .context("Failed to create network connection")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to create connection: {}", error));
        }
        
        let output = Command::new("nmcli")
            .args(&["connection", "up", &connection_name])
            .output()
            .context("Failed to activate network connection")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            self.remove_connection(&connection_name).await.ok();
            return Err(anyhow::anyhow!("Failed to connect: {}", error));
        }
        
        Ok(())
    }

    pub async fn disconnect_from_network(&self) -> Result<()> {
        debug!("Disconnecting from current network");
        
        Command::new("nmcli")
            .args(&["device", "disconnect", &self.interface])
            .output()
            .context("Failed to disconnect from network")?;
        
        Ok(())
    }

    pub async fn get_current_connection(&self) -> Result<Option<String>> {
        let output = Command::new("nmcli")
            .args(&["-t", "-f", "GENERAL.CONNECTION", "dev", "show", &self.interface])
            .output()
            .context("Failed to get current connection")?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if let Some(connection) = line.strip_prefix("GENERAL.CONNECTION:") {
                if !connection.is_empty() && connection != "--" {
                    return Ok(Some(connection.to_string()));
                }
            }
        }
        
        Ok(None)
    }

    pub async fn verify_internet_connection(&self) -> Result<bool> {
        let output = Command::new("ping")
            .args(&["-c", "1", "-W", "2", "8.8.8.8"])
            .output()
            .context("Failed to ping test")?;
        
        Ok(output.status.success())
    }

    async fn remove_connection(&self, connection_name: &str) -> Result<()> {
        Command::new("nmcli")
            .args(&["connection", "delete", connection_name])
            .output()
            .context("Failed to remove connection")?;
        
        Ok(())
    }

    pub async fn get_interface_info(&self) -> Result<InterfaceInfo> {
        let output = Command::new("ip")
            .args(&["addr", "show", &self.interface])
            .output()
            .context("Failed to get interface info")?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut mac_address = String::new();
        let mut ip_address = None;
        
        for line in output_str.lines() {
            if line.contains("link/ether") {
                if let Some(mac) = line.split_whitespace().nth(1) {
                    mac_address = mac.to_string();
                }
            }
            if line.contains("inet ") && !line.contains("127.0.0.1") {
                if let Some(ip) = line.split_whitespace().nth(1) {
                    if let Some(addr) = ip.split('/').next() {
                        ip_address = Some(addr.to_string());
                    }
                }
            }
        }
        
        Ok(InterfaceInfo {
            interface: self.interface.clone(),
            mac_address,
            ip_address,
        })
    }
}

#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    pub interface: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
}

fn parse_security_type(security_str: &str) -> SecurityType {
    let security_lower = security_str.to_lowercase();
    
    if security_lower.contains("wpa3") {
        SecurityType::WPA3
    } else if security_lower.contains("wpa2") {
        SecurityType::WPA2
    } else if security_lower.contains("wpa") {
        SecurityType::WPA
    } else if security_lower.contains("wep") {
        SecurityType::WEP
    } else if security_lower.contains("802.1x") {
        SecurityType::Enterprise
    } else if security_lower == "--" || security_lower == "open" || security_lower.is_empty() {
        SecurityType::Open
    } else {
        SecurityType::WPA2
    }
}

pub async fn list_network_interfaces() -> Result<Vec<String>> {
    let output = Command::new("nmcli")
        .args(&["-t", "-f", "DEVICE,TYPE", "dev", "status"])
        .output()
        .context("Failed to list network interfaces")?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut interfaces = Vec::new();
    
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 && parts[1] == "wifi" {
            interfaces.push(parts[0].to_string());
        }
    }
    
    Ok(interfaces)
}