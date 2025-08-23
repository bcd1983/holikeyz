use anyhow::Result;
use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use env_logger;
use holikeyz::provisioning::{
    elgato::{ElgatoProvisioner, ElgatoLightState, ElgatoLight},
    credential_manager::{CredentialManager, SavedNetwork, get_wifi_credentials},
    WiFiCredentials,
};
use log::{info, error, warn};
use std::time::Duration;
use tokio;

#[derive(Parser)]
#[clap(name = "elgato-provisioner")]
#[clap(about = "Elgato Ring Light Provisioning Tool", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan for Elgato devices in setup mode
    Scan,
    
    /// Interactive provisioning wizard
    Provision {
        /// Auto-connect to first found device
        #[clap(short, long)]
        auto: bool,
    },
    
    /// Control a device on the Elgato's network (for testing)
    Control,
    
    /// Manage saved WiFi credentials
    Credentials {
        #[clap(subcommand)]
        action: CredAction,
    },
    
    /// Quick provision with specific networks
    Quick {
        /// Target WiFi SSID for the device
        #[clap(short, long)]
        ssid: String,
        
        /// WiFi password (will prompt if not provided)
        #[clap(short, long)]
        password: Option<String>,
    },
}

#[derive(Subcommand)]
enum CredAction {
    /// List saved networks
    List,
    
    /// Add a network
    Add {
        ssid: String,
        #[clap(short, long)]
        password: Option<String>,
    },
    
    /// Remove a network
    Remove {
        ssid: String,
    },
    
    /// Clear all saved credentials
    Clear,
}

async fn scan_for_devices() -> Result<Vec<String>> {
    let provisioner = ElgatoProvisioner::new();
    let networks = provisioner.scan_for_elgato_networks().await?;
    
    if networks.is_empty() {
        println!("❌ No Elgato devices found in setup mode");
        println!("\nTo put your device in setup mode:");
        println!("1. Press and hold the button on the back for 10-15 seconds");
        println!("2. The light will blink to indicate setup mode");
        println!("3. Wait for the WiFi network to appear");
        return Ok(vec![]);
    }
    
    println!("🔍 Found {} Elgato device(s):", networks.len());
    let mut device_ssids = Vec::new();
    
    for network in networks {
        println!("   📡 {} (Signal: {}dBm)", network.ssid, network.signal_strength);
        device_ssids.push(network.ssid);
    }
    
    Ok(device_ssids)
}

async fn interactive_provision(auto_connect: bool) -> Result<()> {
    println!("🚀 Elgato Ring Light Provisioning Wizard");
    println!("========================================\n");
    
    // Step 1: Find devices
    println!("Step 1: Scanning for Elgato devices...");
    let devices = scan_for_devices().await?;
    
    if devices.is_empty() {
        return Ok(());
    }
    
    // Step 2: Select device
    let selected_device = if auto_connect || devices.len() == 1 {
        devices[0].clone()
    } else {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select device to provision")
            .items(&devices)
            .interact()?;
        devices[selection].clone()
    };
    
    println!("\n✅ Selected: {}", selected_device);
    
    // Step 3: Connect to device
    println!("\nStep 2: Connecting to device's WiFi...");
    let mut provisioner = ElgatoProvisioner::new();
    
    provisioner.connect_to_elgato(&selected_device).await?;
    println!("✅ Connected to device");
    
    // Get device info
    let device_info = provisioner.get_device_info().await?;
    println!("\n📱 Device Information:");
    println!("   Model: {}", device_info.product_name);
    println!("   Serial: {}", device_info.serial_number);
    println!("   Firmware: {}", device_info.firmware_version);
    println!("   MAC: {}", device_info.mac_address);
    
    // Test light control
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Test light control?")
        .default(true)
        .interact()?
    {
        test_light_control(&provisioner).await?;
    }
    
    // Step 4: Get target network
    println!("\nStep 3: Select target WiFi network");
    
    // Scan for available networks from device
    println!("Scanning for available networks...");
    let available_networks = provisioner.scan_wifi_networks().await?;
    
    if available_networks.is_empty() {
        error!("No WiFi networks found");
        return Ok(());
    }
    
    // Show top networks
    let mut network_names: Vec<String> = available_networks
        .iter()
        .take(10)
        .map(|n| format!("{} ({}dBm)", n.ssid, n.signal_strength))
        .collect();
    network_names.push("Enter manually...".to_string());
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select target network")
        .items(&network_names)
        .interact()?;
    
    let target_ssid = if selection < available_networks.len() {
        available_networks[selection].ssid.clone()
    } else {
        dialoguer::Input::new()
            .with_prompt("Enter SSID")
            .interact()?
    };
    
    // Step 5: Get credentials
    println!("\nStep 4: WiFi Credentials");
    let credentials = get_wifi_credentials(&target_ssid).await?;
    
    // Save credentials if user wants
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Save credentials for future use?")
        .default(true)
        .interact()?
    {
        let manager = CredentialManager::new()?;
        manager.add_network(SavedNetwork {
            ssid: credentials.ssid.clone(),
            password: credentials.password.clone(),
            security_type: format!("{:?}", credentials.security_type),
            last_used: Some(chrono::Utc::now()),
            auto_connect: true,
        })?;
        println!("✅ Credentials saved");
    }
    
    // Step 6: Provision device
    println!("\nStep 5: Sending configuration to device...");
    provisioner.provision_with_credentials(&credentials).await?;
    
    println!("✅ Configuration sent! Device will reboot and connect to '{}'", target_ssid);
    
    // Step 7: Restore original network and wait
    println!("\nStep 6: Restoring your network connection...");
    provisioner.restore_original_network().await?;
    
    // Wait for device to appear on network
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Wait for device to connect to your network?")
        .default(true)
        .interact()?
    {
        println!("Waiting for device to appear on network (up to 60 seconds)...");
        if provisioner.wait_for_device_on_network(60).await? {
            println!("✅ Device successfully connected to your network!");
            
            // Discover and show device details
            let lights = holikeyz::discover_lights(Duration::from_secs(5)).await?;
            for light in lights {
                println!("\n📡 Device found:");
                println!("   Name: {}", light.name);
                println!("   IP: {}", light.ip);
                println!("   Port: {}", light.port);
            }
        } else {
            warn!("⚠️  Device not found on network yet. It may need more time.");
        }
    }
    
    println!("\n✨ Provisioning complete!");
    Ok(())
}

async fn test_light_control(provisioner: &ElgatoProvisioner) -> Result<()> {
    println!("\n🔦 Testing light control...");
    
    // Get current state
    let original_state = provisioner.get_light_state().await?;
    
    // Turn off
    println!("   Turning light off...");
    provisioner.set_light_state(&ElgatoLightState {
        number_of_lights: 1,
        lights: vec![ElgatoLight {
            on: 0,
            brightness: original_state.lights[0].brightness,
            temperature: original_state.lights[0].temperature,
        }],
    }).await?;
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Turn on at 50%
    println!("   Setting to 50% brightness...");
    provisioner.set_light_state(&ElgatoLightState {
        number_of_lights: 1,
        lights: vec![ElgatoLight {
            on: 1,
            brightness: 50,
            temperature: original_state.lights[0].temperature,
        }],
    }).await?;
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Restore original
    println!("   Restoring original state...");
    provisioner.set_light_state(&original_state).await?;
    
    println!("✅ Light control working!");
    Ok(())
}

async fn control_device() -> Result<()> {
    println!("🎮 Elgato Light Control Mode");
    println!("============================\n");
    
    let provisioner = ElgatoProvisioner::new();
    
    // Check if we can reach the device
    match provisioner.get_device_info().await {
        Ok(info) => {
            println!("✅ Connected to: {}", info.product_name);
            println!("   Serial: {}", info.serial_number);
        }
        Err(_) => {
            error!("❌ Cannot reach device at 192.168.62.1:9123");
            println!("Make sure you're connected to the Elgato's WiFi network");
            return Ok(());
        }
    }
    
    loop {
        let options = vec![
            "Get light status",
            "Turn on",
            "Turn off", 
            "Set brightness",
            "Set color temperature",
            "Exit",
        ];
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select action")
            .items(&options)
            .interact()?;
        
        match selection {
            0 => {
                let state = provisioner.get_light_state().await?;
                println!("\nLight Status:");
                for (i, light) in state.lights.iter().enumerate() {
                    println!("  Light {}: {}", i + 1, if light.on == 1 { "ON" } else { "OFF" });
                    println!("    Brightness: {}%", light.brightness);
                    println!("    Temperature: {}K", light.temperature);
                }
            }
            1 => {
                let mut state = provisioner.get_light_state().await?;
                state.lights[0].on = 1;
                provisioner.set_light_state(&state).await?;
                println!("✅ Light turned on");
            }
            2 => {
                let mut state = provisioner.get_light_state().await?;
                state.lights[0].on = 0;
                provisioner.set_light_state(&state).await?;
                println!("✅ Light turned off");
            }
            3 => {
                let brightness: u8 = dialoguer::Input::new()
                    .with_prompt("Brightness (0-100)")
                    .default(50)
                    .interact()?;
                
                let mut state = provisioner.get_light_state().await?;
                state.lights[0].brightness = brightness.min(100);
                provisioner.set_light_state(&state).await?;
                println!("✅ Brightness set to {}%", brightness);
            }
            4 => {
                let temp: u16 = dialoguer::Input::new()
                    .with_prompt("Color temperature (143-344, where 143=7000K, 344=2900K)")
                    .default(230)
                    .interact()?;
                
                let mut state = provisioner.get_light_state().await?;
                state.lights[0].temperature = temp;
                provisioner.set_light_state(&state).await?;
                println!("✅ Temperature set to {}", temp);
            }
            5 => break,
            _ => {}
        }
        
        println!();
    }
    
    Ok(())
}

async fn manage_credentials(action: CredAction) -> Result<()> {
    let manager = CredentialManager::new()?;
    
    match action {
        CredAction::List => {
            let networks = manager.list_networks()?;
            if networks.is_empty() {
                println!("No saved networks");
            } else {
                println!("Saved networks:");
                for network in networks {
                    println!("  • {}", network);
                }
            }
        }
        CredAction::Add { ssid, password } => {
            let password = match password {
                Some(p) => Some(p),
                None => {
                    let use_password = Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Does this network require a password?")
                        .interact()?;
                    
                    if use_password {
                        Some(rpassword::prompt_password("Password: ")?)
                    } else {
                        None
                    }
                }
            };
            
            manager.add_network(SavedNetwork {
                ssid: ssid.clone(),
                password,
                security_type: "WPA2".to_string(),
                last_used: Some(chrono::Utc::now()),
                auto_connect: true,
            })?;
            
            println!("✅ Network '{}' saved", ssid);
        }
        CredAction::Remove { ssid } => {
            manager.remove_network(&ssid)?;
            println!("✅ Network '{}' removed", ssid);
        }
        CredAction::Clear => {
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Clear all saved credentials?")
                .default(false)
                .interact()?
            {
                manager.clear_all()?;
                println!("✅ All credentials cleared");
            }
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan => {
            scan_for_devices().await?;
        }
        Commands::Provision { auto } => {
            interactive_provision(auto).await?;
        }
        Commands::Control => {
            control_device().await?;
        }
        Commands::Credentials { action } => {
            manage_credentials(action).await?;
        }
        Commands::Quick { ssid, password } => {
            println!("Quick provisioning to network: {}", ssid);
            
            // Find device
            let devices = scan_for_devices().await?;
            if devices.is_empty() {
                return Ok(());
            }
            
            let device = &devices[0];
            println!("Using device: {}", device);
            
            // Connect and provision
            let mut provisioner = ElgatoProvisioner::new();
            provisioner.connect_to_elgato(device).await?;
            
            let credentials = WiFiCredentials {
                ssid: ssid.clone(),
                password,
                security_type: holikeyz::provisioning::SecurityType::WPA2,
                hidden: false,
            };
            
            provisioner.provision_with_credentials(&credentials).await?;
            println!("✅ Device configured for network: {}", ssid);
            
            provisioner.restore_original_network().await?;
        }
    }

    Ok(())
}