use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Serialize)]
struct StartProvisioningRequest {
    device_type: String,
    device_ssid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StartProvisioningResponse {
    session_id: String,
    status: String,
    connection_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ScanNetworksResponse {
    networks: Vec<WiFiNetwork>,
}

#[derive(Debug, Deserialize)]
struct WiFiNetwork {
    ssid: String,
    signal_strength: i32,
    security_type: SecurityType,
}

#[derive(Debug, Deserialize)]
enum SecurityType {
    Open,
    WEP,
    WPA,
    WPA2,
    WPA3,
    Enterprise,
}

#[derive(Debug, Serialize)]
struct ProvisioningRequest {
    session_id: String,
    wifi_credentials: WiFiCredentials,
    device_name: Option<String>,
    timezone: Option<String>,
    locale: Option<String>,
}

#[derive(Debug, Serialize)]
struct WiFiCredentials {
    ssid: String,
    password: Option<String>,
    security_type: SecurityType,
    hidden: bool,
}

#[derive(Debug, Deserialize)]
struct ProvisioningResponse {
    session_id: String,
    status: ProvisioningStatus,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
enum ProvisioningStatus {
    Pending,
    Connecting,
    Authenticating,
    Configuring,
    Success,
    Failed,
}

impl Serialize for SecurityType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            SecurityType::Open => "Open",
            SecurityType::WEP => "WEP",
            SecurityType::WPA => "WPA",
            SecurityType::WPA2 => "WPA2",
            SecurityType::WPA3 => "WPA3",
            SecurityType::Enterprise => "Enterprise",
        };
        serializer.serialize_str(s)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let base_url = "http://localhost:9124";
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    
    println!("🚀 Starting Elgato Ring Light Provisioning Demo");
    println!("================================================\n");
    
    println!("Step 1: Detecting Elgato devices nearby");
    
    // First scan to find Elgato devices
    let scan_response: ScanNetworksResponse = client
        .get(format!("{}/provisioning/scan", base_url))
        .send()
        .await?
        .json()
        .await?;
    
    let elgato_network = scan_response.networks.iter()
        .find(|n| n.ssid.starts_with("Elgato Ring Light"))
        .map(|n| n.ssid.clone());
    
    let device_ssid = if let Some(ssid) = elgato_network {
        println!("✅ Found Elgato device: {}", ssid);
        Some(ssid)
    } else {
        println!("⚠️  No Elgato device found in setup mode nearby");
        println!("   Using default: Elgato Ring Light XXXX");
        None
    };
    
    println!("\nStarting provisioning session for Elgato device");
    let start_req = StartProvisioningRequest {
        device_type: "elgato".to_string(),
        device_ssid: device_ssid.clone(),
    };
    
    let response: StartProvisioningResponse = client
        .post(format!("{}/provisioning/start", base_url))
        .json(&start_req)
        .send()
        .await?
        .json()
        .await?;
    
    println!("✅ Session created: {}", response.session_id);
    if let Some(url) = response.connection_url {
        println!("📱 {}", url);
    }
    
    if let Some(ssid) = &device_ssid {
        println!("\n⚠️  Please connect to the device's WiFi network: '{}'", ssid);
    } else {
        println!("\n⚠️  Please ensure your Elgato Ring Light is in setup mode");
        println!("   (Hold the button on the back for 10 seconds until it blinks)");
        println!("   Then connect to its WiFi network (usually 'Elgato Ring Light XXXX')");
    }
    println!("\nPress Enter when connected to the device's WiFi...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    println!("\nStep 2: Scanning for available WiFi networks");
    let scan_response: ScanNetworksResponse = client
        .get(format!("{}/provisioning/scan", base_url))
        .send()
        .await?
        .json()
        .await?;
    
    println!("📡 Found {} networks:", scan_response.networks.len());
    for (i, network) in scan_response.networks.iter().enumerate().take(5) {
        println!("   {}. {} (Signal: {}dBm, Security: {:?})",
            i + 1, network.ssid, network.signal_strength, network.security_type);
    }
    
    println!("\nStep 3: Enter your target WiFi network credentials");
    println!("Network SSID: ");
    let mut ssid = String::new();
    std::io::stdin().read_line(&mut ssid)?;
    let ssid = ssid.trim().to_string();
    
    println!("Network Password (leave empty for open network): ");
    let mut password = String::new();
    std::io::stdin().read_line(&mut password)?;
    let password = password.trim();
    let password = if password.is_empty() { None } else { Some(password.to_string()) };
    
    let security_type = if password.is_none() {
        SecurityType::Open
    } else {
        SecurityType::WPA2
    };
    
    println!("\nStep 4: Provisioning device with WiFi credentials");
    let provision_req = ProvisioningRequest {
        session_id: response.session_id.clone(),
        wifi_credentials: WiFiCredentials {
            ssid: ssid.clone(),
            password,
            security_type,
            hidden: false,
        },
        device_name: Some("My Ring Light".to_string()),
        timezone: Some("America/New_York".to_string()),
        locale: Some("en_US".to_string()),
    };
    
    let provision_response: ProvisioningResponse = client
        .post(format!("{}/provisioning/provision", base_url))
        .json(&provision_req)
        .send()
        .await?
        .json()
        .await?;
    
    println!("📤 Provisioning request sent");
    if let Some(msg) = &provision_response.message {
        println!("   Status: {}", msg);
    }
    
    println!("\n⏳ Waiting for device to connect to network...");
    
    for i in 1..=30 {
        sleep(Duration::from_secs(2)).await;
        
        let status_response: serde_json::Value = client
            .get(format!("{}/provisioning/status/{}", base_url, response.session_id))
            .send()
            .await?
            .json()
            .await?;
        
        if let Some(status) = status_response.get("status").and_then(|s| s.as_str()) {
            print!("\r   Attempt {}/30: Status = {}", i, status);
            
            if status == "Success" {
                println!("\n\n✅ Device successfully provisioned!");
                println!("   The device should now be connected to '{}'", ssid);
                break;
            } else if status == "Failed" {
                println!("\n\n❌ Provisioning failed");
                if let Some(device_info) = status_response.get("device_info") {
                    println!("   Device info: {:?}", device_info);
                }
                break;
            }
        }
    }
    
    println!("\nStep 5: Discovering device on the network");
    sleep(Duration::from_secs(5)).await;
    
    use holikeyz::discovery::discover_lights;
    let lights = discover_lights(Duration::from_secs(10)).await?;
    
    if lights.is_empty() {
        println!("⚠️  No devices found on the network");
        println!("   The device may need more time to connect");
    } else {
        println!("🎉 Found {} device(s) on the network:", lights.len());
        for light in lights {
            println!("   - {} at {}:{}", light.name, light.ip, light.port);
        }
    }
    
    println!("\n✨ Provisioning demo complete!");
    
    Ok(())
}