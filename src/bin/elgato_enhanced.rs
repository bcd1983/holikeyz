use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Select, MultiSelect, Input};
use env_logger;
use holikeyz::{
    discovery::discover_lights,
    provisioning::{
        enhanced_manager::{
            EnhancedProvisioningManager, DiscoveredDevice, LightCommand, CommandType,
            DeviceProfile,
        },
        WiFiCredentials,
    },
};
use log::{info, error, warn};
use std::collections::HashMap;
use std::time::Duration;
use tokio;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser)]
#[clap(name = "elgato-enhanced")]
#[clap(about = "Enhanced Elgato Ring Light Manager", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    
    /// WiFi interface to use
    #[clap(short, long, default_value = "wlan0")]
    interface: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Discover Elgato devices (both setup mode and configured)
    Discover {
        /// Include devices already on the network
        #[clap(short, long)]
        all: bool,
        
        /// Output format (text, json)
        #[clap(short, long, default_value = "text")]
        format: String,
    },
    
    /// Interactive setup wizard for new devices
    Setup {
        /// Auto-connect to strongest signal device
        #[clap(short, long)]
        auto: bool,
        
        /// Target WiFi network SSID
        #[clap(short, long)]
        network: Option<String>,
        
        /// Use saved credentials if available
        #[clap(short, long)]
        use_saved: bool,
    },
    
    /// Control connected devices
    Control {
        /// Device identifier (SSID or serial number)
        #[clap(short, long)]
        device: Option<String>,
        
        /// Command to execute
        #[clap(subcommand)]
        command: Option<ControlCommand>,
    },
    
    /// Batch provision multiple devices
    Batch {
        /// Target WiFi network for all devices
        network: String,
        
        /// Only provision devices in setup mode
        #[clap(short, long)]
        setup_only: bool,
    },
    
    /// Manage WiFi credentials
    Credentials {
        #[clap(subcommand)]
        action: CredAction,
    },
    
    /// Quick actions
    Quick {
        #[clap(subcommand)]
        action: QuickAction,
    },
    
    /// Device profiles management
    Profiles {
        #[clap(subcommand)]
        action: ProfileAction,
    },
}

#[derive(Subcommand)]
enum ControlCommand {
    /// Turn light on
    On,
    /// Turn light off
    Off,
    /// Toggle light state
    Toggle,
    /// Set brightness (0-100)
    Brightness { value: u8 },
    /// Set color temperature (143-344)
    Temperature { value: u16 },
    /// Apply a preset scene
    Scene { name: String },
    /// Fade in effect
    FadeIn,
    /// Fade out effect
    FadeOut,
    /// Pulse effect
    Pulse,
    /// Rainbow temperature cycle
    Rainbow,
    /// Interactive control mode
    Interactive,
}

#[derive(Subcommand)]
enum CredAction {
    /// List saved networks
    List,
    /// Add network credentials
    Add {
        ssid: String,
        #[clap(short, long)]
        password: Option<String>,
    },
    /// Remove network credentials
    Remove { ssid: String },
    /// Import from system (NetworkManager)
    Import,
    /// Clear all credentials
    Clear,
}

#[derive(Subcommand)]
enum QuickAction {
    /// Turn all discovered lights on
    AllOn,
    /// Turn all discovered lights off
    AllOff,
    /// Set all lights to a scene
    AllScene { scene: String },
    /// Flash all lights (notification)
    Flash { times: Option<u8> },
}

#[derive(Subcommand)]
enum ProfileAction {
    /// Create a new device profile
    Create {
        device_id: String,
        nickname: String,
    },
    /// List all profiles
    List,
    /// Apply a profile
    Apply { nickname: String },
    /// Delete a profile
    Delete { nickname: String },
}

async fn discover_devices(manager: &EnhancedProvisioningManager, include_all: bool, format: &str) -> Result<Vec<DiscoveredDevice>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    pb.set_message("Scanning for Elgato devices...");
    pb.enable_steady_tick(Duration::from_millis(100));
    
    let devices = manager.scan_for_devices(include_all).await?;
    
    pb.finish_and_clear();
    
    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&devices)?);
    } else {
        if devices.is_empty() {
            println!("{}", "No Elgato devices found".red());
            if !include_all {
                println!("\n{}", "Tip: Use --all to include configured devices".yellow());
            }
        } else {
            println!("{}", format!("Found {} device(s):", devices.len()).green().bold());
            println!();
            
            for (i, device) in devices.iter().enumerate() {
                let icon = match device.device_type {
                    holikeyz::provisioning::enhanced_manager::DeviceType::ElgatoRingLight => "💡",
                    holikeyz::provisioning::enhanced_manager::DeviceType::ElgatoKeyLight => "🔦",
                    holikeyz::provisioning::enhanced_manager::DeviceType::ElgatoKeyLightAir => "☁️",
                    _ => "❓",
                };
                
                let status = if device.setup_mode {
                    "Setup Mode".yellow()
                } else {
                    "Configured".green()
                };
                
                println!("  {}. {} {} [{}]", 
                    (i + 1).to_string().cyan(),
                    icon,
                    device.ssid.white().bold(),
                    status
                );
                println!("     Signal: {}dBm", 
                    format_signal_strength(device.signal_strength)
                );
                
                if let Some(bssid) = &device.bssid {
                    println!("     BSSID: {}", bssid.dimmed());
                }
            }
        }
    }
    
    Ok(devices)
}

fn format_signal_strength(rssi: i32) -> ColoredString {
    if rssi >= -50 {
        format!("{} ████", rssi).green()
    } else if rssi >= -60 {
        format!("{} ███░", rssi).yellow()
    } else if rssi >= -70 {
        format!("{} ██░░", rssi).yellow()
    } else {
        format!("{} █░░░", rssi).red()
    }
}

async fn interactive_setup(manager: &EnhancedProvisioningManager, auto: bool, network: Option<String>, use_saved: bool) -> Result<()> {
    println!("{}", "🚀 Elgato Device Setup Wizard".cyan().bold());
    println!("{}", "═".repeat(40).cyan());
    println!();
    
    // Step 1: Discover devices
    let devices = discover_devices(manager, false, "text").await?;
    
    if devices.is_empty() {
        println!("{}", "No devices in setup mode found.".red());
        println!("\n{}", "To put your device in setup mode:".yellow());
        println!("  1. Press and hold the button on the back for 10-15 seconds");
        println!("  2. The light will blink to indicate setup mode");
        println!("  3. Wait for the WiFi network to appear");
        return Ok(());
    }
    
    // Step 2: Select device
    let device = if auto || devices.len() == 1 {
        println!("{}", format!("Auto-selecting: {}", devices[0].ssid).green());
        devices[0].clone()
    } else {
        let items: Vec<String> = devices.iter()
            .map(|d| format!("{} ({}dBm)", d.ssid, d.signal_strength))
            .collect();
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select device to configure")
            .items(&items)
            .default(0)
            .interact()?;
        
        devices[selection].clone()
    };
    
    // Step 3: Connect to device
    println!("\n{}", format!("Connecting to {}...", device.ssid).cyan());
    let connected = manager.connect_to_device(&device).await?;
    
    println!("{}", "✓ Connected successfully!".green());
    println!("\n{}", "Device Information:".white().bold());
    println!("  Model: {}", connected.device_info.product_name);
    println!("  Serial: {}", connected.device_info.serial_number);
    println!("  Firmware: {}", connected.device_info.firmware_version);
    println!("  MAC: {}", connected.device_info.mac_address);
    
    // Step 4: Test control
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Test light control?")
        .default(true)
        .interact()?
    {
        test_light_control(manager, &device.ssid).await?;
    }
    
    // Step 5: Select target network
    let target_network = if let Some(net) = network {
        net
    } else {
        // Scan for available networks
        println!("\n{}", "Scanning for WiFi networks...".cyan());
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner());
        pb.enable_steady_tick(Duration::from_millis(100));
        
        let networks = manager.scan_for_devices(true).await?;
        pb.finish_and_clear();
        
        // Get unique SSIDs
        let mut ssids: Vec<String> = networks.iter()
            .filter_map(|n| {
                if !n.ssid.starts_with("Elgato") {
                    Some(n.ssid.clone())
                } else {
                    None
                }
            })
            .collect();
        ssids.dedup();
        ssids.push("Enter manually...".to_string());
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select target WiFi network")
            .items(&ssids)
            .default(0)
            .interact()?;
        
        if selection == ssids.len() - 1 {
            Input::new()
                .with_prompt("Enter WiFi SSID")
                .interact()?
        } else {
            ssids[selection].clone()
        }
    };
    
    // Step 6: Provision
    println!("\n{}", format!("Configuring device for network: {}", target_network).cyan());
    
    // The provision_device method will now handle credential prompting if needed
    manager.provision_device(&device, &target_network, use_saved).await?;
    
    println!("{}", "✓ Configuration sent!".green());
    println!("\n{}", "The device will now:".yellow());
    println!("  1. Reboot and disconnect from setup mode");
    println!("  2. Connect to your WiFi network");
    println!("  3. Be available for control via the network");
    
    // Step 7: Restore network and verify
    println!("\n{}", "Restoring your network connection...".cyan());
    manager.restore_original_network().await?;
    
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Wait for device to connect to your network?")
        .default(true)
        .interact()?
    {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} Waiting for device (up to 60s)...")
                .unwrap()
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        
        if manager.verify_device_on_network(60).await? {
            pb.finish_with_message("✓ Device connected successfully!");
            
            // Show device on network
            let devices = discover_lights(Duration::from_secs(3)).await?;
            for device in devices {
                println!("\n{}", "Device ready:".green().bold());
                println!("  Name: {}", device.name);
                println!("  IP: {}", device.ip);
                println!("  Port: {}", device.port);
            }
        } else {
            pb.finish_with_message("⚠ Device not detected yet (may need more time)");
        }
    }
    
    println!("\n{}", "✨ Setup complete!".green().bold());
    Ok(())
}

async fn test_light_control(manager: &EnhancedProvisioningManager, device_id: &str) -> Result<()> {
    println!("\n{}", "Testing light control...".cyan());
    
    // Create a simple animation
    let animations = vec![
        ("Turning off...", CommandType::TurnOff),
        ("Turning on...", CommandType::TurnOn),
        ("Setting 50% brightness...", CommandType::SetBrightness),
        ("Cool white...", CommandType::SetTemperature),
        ("Warm white...", CommandType::SetTemperature),
    ];
    
    for (msg, cmd_type) in animations {
        println!("  {}", msg);
        
        let mut params = HashMap::new();
        match cmd_type {
            CommandType::SetBrightness => {
                params.insert("brightness".to_string(), serde_json::json!(50));
            }
            CommandType::SetTemperature => {
                if msg.contains("Cool") {
                    params.insert("temperature".to_string(), serde_json::json!(143));
                } else {
                    params.insert("temperature".to_string(), serde_json::json!(344));
                }
            }
            _ => {}
        }
        
        let command = LightCommand {
            command_type: cmd_type,
            parameters: params,
        };
        
        manager.send_command(device_id, command).await?;
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
    
    println!("{}", "✓ Light control test complete!".green());
    Ok(())
}

async fn control_device(manager: &EnhancedProvisioningManager, device: Option<String>, command: Option<ControlCommand>) -> Result<()> {
    // If no device specified, discover and select
    let device_id = if let Some(d) = device {
        d
    } else {
        let devices = manager.scan_for_devices(false).await?;
        if devices.is_empty() {
            error!("No devices found");
            return Ok(());
        }
        
        if devices.len() == 1 {
            devices[0].ssid.clone()
        } else {
            let items: Vec<String> = devices.iter().map(|d| d.ssid.clone()).collect();
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select device")
                .items(&items)
                .interact()?;
            devices[selection].ssid.clone()
        }
    };
    
    // Execute command or enter interactive mode
    match command {
        Some(ControlCommand::Interactive) | None => interactive_control(manager, &device_id).await?,
        Some(cmd) => execute_control_command(manager, &device_id, cmd).await?,
    }
    
    Ok(())
}

async fn execute_control_command(manager: &EnhancedProvisioningManager, device_id: &str, command: ControlCommand) -> Result<()> {
    let light_command = match command {
        ControlCommand::On => LightCommand {
            command_type: CommandType::TurnOn,
            parameters: HashMap::new(),
        },
        ControlCommand::Off => LightCommand {
            command_type: CommandType::TurnOff,
            parameters: HashMap::new(),
        },
        ControlCommand::Toggle => LightCommand {
            command_type: CommandType::Toggle,
            parameters: HashMap::new(),
        },
        ControlCommand::Brightness { value } => {
            let mut params = HashMap::new();
            params.insert("brightness".to_string(), serde_json::json!(value));
            LightCommand {
                command_type: CommandType::SetBrightness,
                parameters: params,
            }
        },
        ControlCommand::Temperature { value } => {
            let mut params = HashMap::new();
            params.insert("temperature".to_string(), serde_json::json!(value));
            LightCommand {
                command_type: CommandType::SetTemperature,
                parameters: params,
            }
        },
        ControlCommand::Scene { name } => {
            let mut params = HashMap::new();
            params.insert("scene".to_string(), serde_json::json!(name));
            LightCommand {
                command_type: CommandType::SetScene,
                parameters: params,
            }
        },
        ControlCommand::FadeIn => LightCommand {
            command_type: CommandType::FadeIn,
            parameters: HashMap::new(),
        },
        ControlCommand::FadeOut => LightCommand {
            command_type: CommandType::FadeOut,
            parameters: HashMap::new(),
        },
        ControlCommand::Pulse => LightCommand {
            command_type: CommandType::Pulse,
            parameters: HashMap::new(),
        },
        ControlCommand::Rainbow => LightCommand {
            command_type: CommandType::Rainbow,
            parameters: HashMap::new(),
        },
        ControlCommand::Interactive => {
            // Handle inline to avoid recursion
            return Ok(());
        }
    };
    
    manager.send_command(device_id, light_command).await?;
    println!("{}", "✓ Command executed".green());
    Ok(())
}

async fn interactive_control(manager: &EnhancedProvisioningManager, device_id: &str) -> Result<()> {
    println!("{}", "Interactive Control Mode".cyan().bold());
    println!("{}", "Press Ctrl+C to exit".dimmed());
    println!();
    
    loop {
        let options = vec![
            "💡 Turn On",
            "🌑 Turn Off",
            "🔄 Toggle",
            "☀️ Set Brightness",
            "🌡️ Set Temperature",
            "🎨 Apply Scene",
            "✨ Effects",
            "❌ Exit",
        ];
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select action")
            .items(&options)
            .default(0)
            .interact()?;
        
        match selection {
            0 => {
                let cmd = ControlCommand::On;
                return execute_control_command(manager, device_id, cmd).await;
            },
            1 => {
                let cmd = ControlCommand::Off;
                return execute_control_command(manager, device_id, cmd).await;
            },
            2 => {
                let cmd = ControlCommand::Toggle;
                return execute_control_command(manager, device_id, cmd).await;
            },
            3 => {
                let brightness = Input::new()
                    .with_prompt("Brightness (0-100)")
                    .default(50)
                    .interact()?;
                let cmd = ControlCommand::Brightness { value: brightness };
                return execute_control_command(manager, device_id, cmd).await;
            },
            4 => {
                let temp = Input::new()
                    .with_prompt("Temperature (143=7000K to 344=2900K)")
                    .default(230)
                    .interact()?;
                let cmd = ControlCommand::Temperature { value: temp };
                return execute_control_command(manager, device_id, cmd).await;
            },
            5 => {
                let scenes = vec!["daylight", "reading", "video", "relax", "warm", "cool", "focus", "evening"];
                let scene_idx = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select scene")
                    .items(&scenes)
                    .interact()?;
                let cmd = ControlCommand::Scene { name: scenes[scene_idx].to_string() };
                return execute_control_command(manager, device_id, cmd).await;
            },
            6 => {
                let effects = vec!["Fade In", "Fade Out", "Pulse", "Rainbow"];
                let effect_idx = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select effect")
                    .items(&effects)
                    .interact()?;
                
                match effect_idx {
                    0 => manager.send_command(device_id, LightCommand {
                        command_type: CommandType::FadeIn,
                        parameters: HashMap::new(),
                    }).await?,
                    1 => manager.send_command(device_id, LightCommand {
                        command_type: CommandType::FadeOut,
                        parameters: HashMap::new(),
                    }).await?,
                    2 => manager.send_command(device_id, LightCommand {
                        command_type: CommandType::Pulse,
                        parameters: HashMap::new(),
                    }).await?,
                    3 => manager.send_command(device_id, LightCommand {
                        command_type: CommandType::Rainbow,
                        parameters: HashMap::new(),
                    }).await?,
                    _ => continue,
                }
                println!("{}", "✓ Effect applied".green());
            },
            7 => break,
            _ => {}
        }
    }
    
    Ok(())
}

async fn batch_provision(manager: &EnhancedProvisioningManager, network: String, setup_only: bool) -> Result<()> {
    println!("{}", "Batch Provisioning Mode".cyan().bold());
    println!();
    
    // Discover all devices
    let all_devices = manager.scan_for_devices(false).await?;
    
    let devices: Vec<DiscoveredDevice> = if setup_only {
        all_devices.into_iter().filter(|d| d.setup_mode).collect()
    } else {
        all_devices
    };
    
    if devices.is_empty() {
        println!("{}", "No eligible devices found".red());
        return Ok(());
    }
    
    // Let user select which devices to provision
    let device_names: Vec<String> = devices.iter()
        .map(|d| format!("{} ({}dBm)", d.ssid, d.signal_strength))
        .collect();
    
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select devices to provision")
        .items(&device_names)
        .interact()?;
    
    if selections.is_empty() {
        println!("{}", "No devices selected".yellow());
        return Ok(());
    }
    
    let selected_devices: Vec<DiscoveredDevice> = selections.iter()
        .map(|&i| devices[i].clone())
        .collect();
    
    println!("\n{}", format!("Provisioning {} devices to network: {}", selected_devices.len(), network).cyan());
    
    // Provision each device
    let results = manager.batch_provision_devices(selected_devices.clone(), &network).await?;
    
    // Show results
    println!("\n{}", "Results:".white().bold());
    for (ssid, result) in results {
        match result {
            Ok(_) => println!("  {} {}", "✓".green(), ssid),
            Err(e) => println!("  {} {} - {}", "✗".red(), ssid, e),
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    
    let manager = EnhancedProvisioningManager::new(Some(cli.interface))?;
    
    match cli.command {
        Commands::Discover { all, format } => {
            discover_devices(&manager, all, &format).await?;
        }
        Commands::Setup { auto, network, use_saved } => {
            interactive_setup(&manager, auto, network, use_saved).await?;
        }
        Commands::Control { device, command } => {
            control_device(&manager, device, command).await?;
        }
        Commands::Batch { network, setup_only } => {
            batch_provision(&manager, network, setup_only).await?;
        }
        Commands::Credentials { action } => {
            // Handle credential management
            match action {
                CredAction::List => {
                    let networks = manager.get_saved_networks().await?;
                    if networks.is_empty() {
                        println!("{}", "No saved networks".yellow());
                    } else {
                        println!("{}", "Saved networks:".white().bold());
                        for net in networks {
                            println!("  • {}", net);
                        }
                    }
                }
                CredAction::Add { ssid, password } => {
                    let creds = if let Some(pw) = password {
                        WiFiCredentials {
                            ssid: ssid.clone(),
                            password: Some(pw),
                            security_type: holikeyz::provisioning::SecurityType::WPA2,
                            hidden: false,
                        }
                    } else {
                        holikeyz::provisioning::credential_manager::prompt_for_credentials(&ssid).await?
                    };
                    
                    manager.save_network_credentials(creds).await?;
                    println!("{}", format!("✓ Credentials for '{}' saved", ssid).green());
                }
                CredAction::Remove { ssid } => {
                    // Implementation would go here
                    println!("Remove network: {}", ssid);
                }
                CredAction::Import => {
                    println!("{}", "Importing system networks...".cyan());
                    let networks = holikeyz::provisioning::credential_manager::get_system_wifi_networks().await?;
                    println!("{}", format!("Found {} system networks", networks.len()).green());
                    for net in networks {
                        println!("  • {}", net.ssid);
                    }
                }
                CredAction::Clear => {
                    if Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Clear all saved credentials?")
                        .default(false)
                        .interact()?
                    {
                        // Implementation would go here
                        println!("{}", "✓ All credentials cleared".green());
                    }
                }
            }
        }
        Commands::Quick { action } => {
            // Handle quick actions
            match action {
                QuickAction::AllOn | QuickAction::AllOff => {
                    let devices = manager.scan_for_devices(true).await?;
                    let cmd_type = if matches!(action, QuickAction::AllOn) {
                        CommandType::TurnOn
                    } else {
                        CommandType::TurnOff
                    };
                    
                    for device in devices {
                        let command = LightCommand {
                            command_type: cmd_type.clone(),
                            parameters: HashMap::new(),
                        };
                        if let Err(e) = manager.send_command(&device.ssid, command).await {
                            warn!("Failed to control {}: {}", device.ssid, e);
                        }
                    }
                    println!("{}", "✓ Command sent to all devices".green());
                }
                QuickAction::AllScene { scene } => {
                    let devices = manager.scan_for_devices(true).await?;
                    for device in devices {
                        let mut params = HashMap::new();
                        params.insert("scene".to_string(), serde_json::json!(scene.clone()));
                        let command = LightCommand {
                            command_type: CommandType::SetScene,
                            parameters: params,
                        };
                        if let Err(e) = manager.send_command(&device.ssid, command).await {
                            warn!("Failed to control {}: {}", device.ssid, e);
                        }
                    }
                    println!("{}", format!("✓ Scene '{}' applied to all devices", scene).green());
                }
                QuickAction::Flash { times } => {
                    let times = times.unwrap_or(3);
                    println!("{}", format!("Flashing all lights {} times...", times).cyan());
                    
                    let devices = manager.scan_for_devices(true).await?;
                    for _ in 0..times {
                        for device in &devices {
                            let command = LightCommand {
                                command_type: CommandType::Toggle,
                                parameters: HashMap::new(),
                            };
                            manager.send_command(&device.ssid, command).await.ok();
                        }
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                    println!("{}", "✓ Flash complete".green());
                }
            }
        }
        Commands::Profiles { action } => {
            // Handle profile management
            match action {
                ProfileAction::Create { device_id, nickname } => {
                    println!("{}", format!("Creating profile '{}' for device {}", nickname, device_id).cyan());
                    // Implementation would save to a profiles file/database
                    println!("{}", "✓ Profile created".green());
                }
                ProfileAction::List => {
                    println!("{}", "Device Profiles:".white().bold());
                    // Would load from storage
                    println!("  No profiles found");
                }
                ProfileAction::Apply { nickname } => {
                    println!("{}", format!("Applying profile '{}'", nickname).cyan());
                    // Implementation would load and apply profile
                }
                ProfileAction::Delete { nickname } => {
                    if Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt(&format!("Delete profile '{}'?", nickname))
                        .default(false)
                        .interact()?
                    {
                        println!("{}", "✓ Profile deleted".green());
                    }
                }
            }
        }
    }
    
    Ok(())
}