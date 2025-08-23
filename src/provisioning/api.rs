use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{Filter, Rejection, Reply};
use log::{info, debug, warn};

use crate::provisioning::{
    ProvisioningManager, ProvisioningRequest, ProvisioningResponse,
    ProvisioningStatus, WiFiNetwork, DeviceInfo,
    device_provisioner::{ElgatoProvisioner, GenericProvisioner},
    ProvisioningService,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartProvisioningRequest {
    pub device_type: String,
    pub device_ssid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartProvisioningResponse {
    pub session_id: String,
    pub status: String,
    pub connection_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanNetworksResponse {
    pub networks: Vec<WiFiNetwork>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatusResponse {
    pub session_id: String,
    pub status: ProvisioningStatus,
    pub device_info: Option<DeviceInfo>,
}

pub struct ProvisioningAPI {
    manager: Arc<ProvisioningManager>,
    provisioner: Arc<RwLock<Box<dyn ProvisioningService>>>,
}

impl ProvisioningAPI {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(ProvisioningManager::new()),
            provisioner: Arc::new(RwLock::new(Box::new(ElgatoProvisioner::new()))),
        }
    }

    pub fn routes(self: Arc<Self>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let api = self.clone();
        let start_provisioning = warp::path!("provisioning" / "start")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(handle_start_provisioning);

        let api = self.clone();
        let scan_networks = warp::path!("provisioning" / "scan")
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(handle_scan_networks);

        let api = self.clone();
        let provision_device = warp::path!("provisioning" / "provision")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(handle_provision_device);

        let api = self.clone();
        let get_session_status = warp::path!("provisioning" / "status" / String)
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(handle_get_session_status);

        let api = self.clone();
        let stop_provisioning = warp::path!("provisioning" / "stop" / String)
            .and(warp::post())
            .and(with_api(api.clone()))
            .and_then(handle_stop_provisioning);

        start_provisioning
            .or(scan_networks)
            .or(provision_device)
            .or(get_session_status)
            .or(stop_provisioning)
    }
}

fn with_api(
    api: Arc<ProvisioningAPI>,
) -> impl Filter<Extract = (Arc<ProvisioningAPI>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || api.clone())
}

async fn handle_start_provisioning(
    req: StartProvisioningRequest,
    api: Arc<ProvisioningAPI>,
) -> Result<impl Reply, Rejection> {
    info!("Starting provisioning for device type: {}", req.device_type);

    let provisioner: Box<dyn ProvisioningService> = match req.device_type.as_str() {
        "elgato" | "ring_light" => Box::new(ElgatoProvisioner::new()),
        _ => Box::new(GenericProvisioner::new()),
    };

    *api.provisioner.write().await = provisioner;

    let device_info = api.provisioner.read().await
        .get_device_info()
        .await
        .unwrap_or_else(|_| DeviceInfo {
            device_id: "unknown".to_string(),
            device_type: req.device_type.clone(),
            manufacturer: "Unknown".to_string(),
            model: "Unknown".to_string(),
            firmware_version: "Unknown".to_string(),
            mac_address: "00:00:00:00:00:00".to_string(),
            capabilities: vec![],
        });

    let session_id = api.manager
        .create_session(device_info, std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 4, 1)))
        .await
        .map_err(|e| {
            warn!("Failed to create session: {}", e);
            warp::reject::reject()
        })?;

    let response = StartProvisioningResponse {
        session_id,
        status: "ready".to_string(),
        connection_url: req.device_ssid.map(|ssid| {
            format!("Connect to WiFi network: {}", ssid)
        }),
    };

    Ok(warp::reply::json(&response))
}

async fn handle_scan_networks(
    api: Arc<ProvisioningAPI>,
) -> Result<impl Reply, Rejection> {
    debug!("Scanning for WiFi networks");

    let networks = api.provisioner.read().await
        .scan_wifi_networks()
        .await
        .map_err(|e| {
            warn!("Failed to scan networks: {}", e);
            warp::reject::reject()
        })?;

    let response = ScanNetworksResponse { networks };
    Ok(warp::reply::json(&response))
}

async fn handle_provision_device(
    req: ProvisioningRequest,
    api: Arc<ProvisioningAPI>,
) -> Result<impl Reply, Rejection> {
    info!("Provisioning device with session: {}", req.session_id);

    api.manager
        .update_session_status(&req.session_id, ProvisioningStatus::Connecting)
        .await
        .map_err(|e| {
            warn!("Failed to update session: {}", e);
            warp::reject::reject()
        })?;

    let response = api.provisioner.read().await
        .provision_device(req.clone())
        .await
        .map_err(|e| {
            warn!("Provisioning failed: {}", e);
            warp::reject::reject()
        })?;

    api.manager
        .update_session_status(&req.session_id, response.status.clone())
        .await
        .ok();

    if matches!(response.status, ProvisioningStatus::Success) {
        api.manager.complete_session(&req.session_id).await.ok();
    }

    Ok(warp::reply::json(&response))
}

async fn handle_get_session_status(
    session_id: String,
    api: Arc<ProvisioningAPI>,
) -> Result<impl Reply, Rejection> {
    let session = api.manager
        .get_session(&session_id)
        .await
        .map_err(|e| {
            warn!("Failed to get session: {}", e);
            warp::reject::reject()
        })?
        .ok_or_else(|| {
            warn!("Session not found: {}", session_id);
            warp::reject::reject()
        })?;

    let response = SessionStatusResponse {
        session_id: session.id,
        status: session.status,
        device_info: Some(session.device_info),
    };

    Ok(warp::reply::json(&response))
}

async fn handle_stop_provisioning(
    session_id: String,
    api: Arc<ProvisioningAPI>,
) -> Result<impl Reply, Rejection> {
    info!("Stopping provisioning session: {}", session_id);

    api.provisioner.read().await
        .stop_soft_ap()
        .await
        .ok();

    api.manager
        .complete_session(&session_id)
        .await
        .map_err(|e| {
            warn!("Failed to complete session: {}", e);
            warp::reject::reject()
        })?;

    Ok(warp::reply::json(&serde_json::json!({
        "status": "stopped",
        "session_id": session_id
    })))
}

pub async fn start_provisioning_server(port: u16) -> Result<()> {
    let api = Arc::new(ProvisioningAPI::new());
    let routes = api.routes();

    info!("Starting provisioning API server on port {}", port);

    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;

    Ok(())
}