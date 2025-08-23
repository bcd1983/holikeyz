use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub mod soft_ap;
pub mod wifi_manager;
pub mod device_provisioner;
pub mod security;
pub mod api;
pub mod elgato;
pub mod elgato_fixed;
pub mod elgato_web;
pub mod credential_manager;
pub mod enhanced_manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_type: String,
    pub manufacturer: String,
    pub model: String,
    pub firmware_version: String,
    pub mac_address: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiNetwork {
    pub ssid: String,
    pub bssid: Option<String>,
    pub signal_strength: i32,
    pub security_type: SecurityType,
    pub frequency: u32,
    pub channel: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityType {
    Open,
    WEP,
    WPA,
    WPA2,
    WPA3,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiCredentials {
    pub ssid: String,
    pub password: Option<String>,
    pub security_type: SecurityType,
    pub hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningRequest {
    pub session_id: String,
    pub wifi_credentials: WiFiCredentials,
    pub device_name: Option<String>,
    pub timezone: Option<String>,
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningResponse {
    pub session_id: String,
    pub status: ProvisioningStatus,
    pub message: Option<String>,
    pub device_info: Option<DeviceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProvisioningStatus {
    Pending,
    Connecting,
    Authenticating,
    Configuring,
    Success,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ProvisioningSession {
    pub id: String,
    pub device_info: DeviceInfo,
    pub started_at: DateTime<Utc>,
    pub status: ProvisioningStatus,
    pub soft_ap_ip: IpAddr,
    pub target_network: Option<WiFiCredentials>,
}

#[async_trait]
pub trait ProvisioningService: Send + Sync {
    async fn start_soft_ap(&self, ssid: &str, password: Option<&str>) -> Result<IpAddr>;
    async fn stop_soft_ap(&self) -> Result<()>;
    async fn scan_wifi_networks(&self) -> Result<Vec<WiFiNetwork>>;
    async fn provision_device(&self, request: ProvisioningRequest) -> Result<ProvisioningResponse>;
    async fn get_device_info(&self) -> Result<DeviceInfo>;
    async fn verify_connection(&self) -> Result<bool>;
}

pub struct ProvisioningManager {
    sessions: Arc<RwLock<HashMap<String, ProvisioningSession>>>,
    current_session: Arc<RwLock<Option<String>>>,
}

impl ProvisioningManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            current_session: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn create_session(&self, device_info: DeviceInfo, soft_ap_ip: IpAddr) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        let session = ProvisioningSession {
            id: session_id.clone(),
            device_info,
            started_at: Utc::now(),
            status: ProvisioningStatus::Pending,
            soft_ap_ip,
            target_network: None,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        
        let mut current = self.current_session.write().await;
        *current = Some(session_id.clone());

        Ok(session_id)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<ProvisioningSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    pub async fn update_session_status(&self, session_id: &str, status: ProvisioningStatus) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = status;
            Ok(())
        } else {
            anyhow::bail!("Session not found: {}", session_id)
        }
    }

    pub async fn complete_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        
        let mut current = self.current_session.write().await;
        if current.as_ref() == Some(&session_id.to_string()) {
            *current = None;
        }
        
        Ok(())
    }
}