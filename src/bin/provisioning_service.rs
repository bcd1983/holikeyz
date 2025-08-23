use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger;
use holikeyz::provisioning::{
    device_provisioner::{ElgatoProvisioner, GenericProvisioner},
    ProvisioningManager, ProvisioningRequest, WiFiCredentials, SecurityType,
    ProvisioningService,
};
use log::{info, error};
use std::time::Duration;
use tokio;

#[derive(Parser)]
#[clap(name = "holikeyz-provisioning")]
#[clap(about = "Holikeyz Device Provisioning Service", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Server {
        #[clap(short, long, default_value = "9124")]
        port: u16,
    },
    
    Provision {
        #[clap(short = 't', long, default_value = "elgato")]
        device_type: String,
        
        #[clap(short = 'd', long)]
        device_ssid: Option<String>,
        
        #[clap(short = 's', long)]
        target_ssid: String,
        
        #[clap(short = 'p', long)]
        target_password: Option<String>,
        
        #[clap(long)]
        open_network: bool,
    },
    
    Scan {
        #[clap(short, long, default_value = "wlan0")]
        interface: String,
    },
    
    Discover {
        #[clap(short, long, default_value = "30")]
        timeout: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { port } => {
            info!("Starting provisioning API server on port {}", port);
            holikeyz::provisioning::api::start_provisioning_server(port).await?;
        }
        
        Commands::Provision {
            device_type,
            device_ssid,
            target_ssid,
            target_password,
            open_network,
        } => {
            info!("Starting device provisioning");
            
            let provisioner: Box<dyn ProvisioningService> = match device_type.as_str() {
                "elgato" | "ring_light" => {
                    info!("Using Elgato provisioner");
                    Box::new(ElgatoProvisioner::new())
                }
                _ => {
                    info!("Using generic provisioner");
                    Box::new(GenericProvisioner::new())
                }
            };
            
            if let Some(device_ap) = device_ssid {
                info!("Connect to device's Soft AP: {}", device_ap);
                info!("This is typically done automatically when the device is in setup mode");
            }
            
            let security_type = if open_network {
                SecurityType::Open
            } else if target_password.is_some() {
                SecurityType::WPA2
            } else {
                error!("Password required for secured network");
                return Ok(());
            };
            
            let credentials = WiFiCredentials {
                ssid: target_ssid.clone(),
                password: target_password,
                security_type,
                hidden: false,
            };
            
            let manager = ProvisioningManager::new();
            let device_info = provisioner.get_device_info().await?;
            let session_id = manager.create_session(
                device_info,
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 4, 1))
            ).await?;
            
            info!("Created provisioning session: {}", session_id);
            
            let request = ProvisioningRequest {
                session_id: session_id.clone(),
                wifi_credentials: credentials,
                device_name: Some("My Ring Light".to_string()),
                timezone: Some("America/New_York".to_string()),
                locale: Some("en_US".to_string()),
            };
            
            info!("Provisioning device...");
            match provisioner.provision_device(request).await {
                Ok(response) => {
                    info!("Provisioning status: {:?}", response.status);
                    if let Some(msg) = response.message {
                        info!("Message: {}", msg);
                    }
                }
                Err(e) => {
                    error!("Provisioning failed: {}", e);
                }
            }
        }
        
        Commands::Scan { interface } => {
            info!("Scanning WiFi networks on interface {}", interface);
            
            let wifi_manager = holikeyz::provisioning::wifi_manager::WiFiManager::new(interface);
            let networks = wifi_manager.scan_networks().await?;
            
            info!("Found {} networks:", networks.len());
            for network in networks {
                println!(
                    "  {} - Signal: {}dBm, Security: {:?}, Channel: {}",
                    network.ssid,
                    network.signal_strength,
                    network.security_type,
                    network.channel
                );
            }
        }
        
        Commands::Discover { timeout } => {
            info!("Discovering devices for {} seconds", timeout);
            
            let lights = holikeyz::discover_lights(Duration::from_secs(timeout)).await?;
            
            if lights.is_empty() {
                info!("No devices found");
            } else {
                info!("Found {} device(s):", lights.len());
                for light in lights {
                    println!("  {} - {}:{}", light.name, light.ip, light.port);
                }
            }
        }
    }

    Ok(())
}