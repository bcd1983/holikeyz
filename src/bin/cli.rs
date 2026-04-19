use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use holikeyz::{RingLightClient, discover_lights, models::*, resolve_active};
use env_logger;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "holikeyz-cli")]
#[command(about = "Control Ring Light via command line (Holikeyz - unofficial open source controller)", long_about = None)]
struct Cli {
    /// Light IP address. Falls back to ~/.config/holikeyz/active.json or $RING_LIGHT_IP.
    #[arg(short, long)]
    ip: Option<String>,

    /// Light port (protocol default 9123).
    #[arg(short, long)]
    port: Option<u16>,

    #[command(subcommand)]
    command: Commands,
}

fn build_client(ip: Option<&str>, port: Option<u16>) -> Result<RingLightClient> {
    let active = resolve_active(ip, port).context(
        "No Ring Light configured. Run `holikeyz-cli discover` to find one, \
         then pass --ip <ip>, or set RING_LIGHT_IP, or select one from the \
         KDE plasmoid / GNOME extension (which persists to ~/.config/holikeyz/active.json).",
    )?;
    Ok(RingLightClient::new(&active.ip, active.port))
}

#[derive(Subcommand)]
enum Commands {
    On,
    
    Off,
    
    Toggle,
    
    Brightness {
        #[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
        value: u8,
    },
    
    Temperature {
        #[arg(help = "Temperature in Kelvin (2900-7000)")]
        #[arg(value_parser = clap::value_parser!(u32).range(2900..=7000))]
        kelvin: u32,
    },
    
    Status,
    
    Info,
    
    Settings,
    
    SetSettings {
        #[arg(long)]
        power_on_behavior: Option<u8>,
        #[arg(long)]
        power_on_brightness: Option<u8>,
        #[arg(long)]
        power_on_temperature: Option<u16>,
        #[arg(long)]
        switch_on_duration: Option<u32>,
        #[arg(long)]
        switch_off_duration: Option<u32>,
        #[arg(long)]
        color_change_duration: Option<u32>,
    },
    
    Identify,
    
    Discover {
        #[arg(short, long, default_value = "5")]
        timeout: u64,
    },
    
    Scene {
        #[command(subcommand)]
        scene: Scene,
    },
}

#[derive(Subcommand)]
enum Scene {
    Daylight,
    
    Warm,
    
    Cool,
    
    Reading,
    
    Video,
    
    Relax,
    
    Custom {
        brightness: u8,
        kelvin: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Discover { timeout } => {
            println!("Discovering Ring Lights on the network...");
            let lights = discover_lights(Duration::from_secs(timeout)).await?;
            
            if lights.is_empty() {
                println!("No Ring Lights found on the network.");
            } else {
                println!("Found {} light(s):", lights.len());
                for light in lights {
                    println!("  - {} at {}:{}", light.name, light.ip, light.port);
                }
            }
        }
        _ => {
            let client = build_client(cli.ip.as_deref(), cli.port)?;
            
            match cli.command {
                Commands::On => {
                    client.turn_on().await?;
                    println!("Light turned on");
                }
                
                Commands::Off => {
                    client.turn_off().await?;
                    println!("Light turned off");
                }
                
                Commands::Toggle => {
                    let current = client.get_lights().await?;
                    if let Some(light) = current.lights.first() {
                        if light.is_on() {
                            client.turn_off().await?;
                            println!("Light turned off");
                        } else {
                            client.turn_on().await?;
                            println!("Light turned on");
                        }
                    }
                }
                
                Commands::Brightness { value } => {
                    client.set_brightness(value).await?;
                    println!("Brightness set to {}%", value);
                }
                
                Commands::Temperature { kelvin } => {
                    client.set_temperature_kelvin(kelvin).await?;
                    println!("Temperature set to {}K", kelvin);
                }
                
                Commands::Status => {
                    let response = client.get_lights().await?;
                    if let Some(light) = response.lights.first() {
                        println!("Light Status:");
                        println!("  State: {}", if light.is_on() { "ON" } else { "OFF" });
                        println!("  Brightness: {}%", light.brightness);
                        println!("  Temperature: {}K", LightState::api_to_kelvin(light.temperature));
                        println!("  Temperature (API): {}", light.temperature);
                    }
                }
                
                Commands::Info => {
                    let info = client.get_accessory_info().await?;
                    println!("Accessory Information:");
                    println!("  Product: {}", info.product_name);
                    println!("  Display Name: {}", info.display_name);
                    println!("  Serial Number: {}", info.serial_number);
                    println!("  Firmware: {} (build {})", info.firmware_version, info.firmware_build_number);
                    println!("  Hardware Board: {}", info.hardware_board_type);
                    println!("  Features: {:?}", info.features);
                }
                
                Commands::Settings => {
                    let settings = client.get_settings().await?;
                    println!("Device Settings:");
                    println!("  Power On Behavior: {}", settings.power_on_behavior);
                    println!("  Power On Brightness: {}%", settings.power_on_brightness);
                    println!("  Power On Temperature: {}K", LightState::api_to_kelvin(settings.power_on_temperature));
                    println!("  Switch On Duration: {}ms", settings.switch_on_duration_ms);
                    println!("  Switch Off Duration: {}ms", settings.switch_off_duration_ms);
                    println!("  Color Change Duration: {}ms", settings.color_change_duration_ms);
                }
                
                Commands::SetSettings { 
                    power_on_behavior,
                    power_on_brightness,
                    power_on_temperature,
                    switch_on_duration,
                    switch_off_duration,
                    color_change_duration
                } => {
                    let mut settings = client.get_settings().await?;
                    
                    if let Some(v) = power_on_behavior {
                        settings.power_on_behavior = v;
                    }
                    if let Some(v) = power_on_brightness {
                        settings.power_on_brightness = v;
                    }
                    if let Some(v) = power_on_temperature {
                        settings.power_on_temperature = v;
                    }
                    if let Some(v) = switch_on_duration {
                        settings.switch_on_duration_ms = v;
                    }
                    if let Some(v) = switch_off_duration {
                        settings.switch_off_duration_ms = v;
                    }
                    if let Some(v) = color_change_duration {
                        settings.color_change_duration_ms = v;
                    }
                    
                    client.set_settings(&settings).await?;
                    println!("Settings updated successfully");
                }
                
                Commands::Identify => {
                    client.identify().await?;
                    println!("Light should be flashing now");
                }
                
                Commands::Scene { scene } => {
                    let (brightness, kelvin) = match scene {
                        Scene::Daylight => (80, 5600),
                        Scene::Warm => (60, 3200),
                        Scene::Cool => (70, 6500),
                        Scene::Reading => (90, 4500),
                        Scene::Video => (75, 5000),
                        Scene::Relax => (40, 2900),
                        Scene::Custom { brightness, kelvin } => (brightness, kelvin),
                    };
                    
                    let mut state = client.get_lights().await?.lights.first().cloned()
                        .unwrap_or_else(LightState::new);
                    
                    state.set_on(true);
                    state.brightness = brightness;
                    state.temperature = LightState::kelvin_to_api(kelvin);
                    
                    client.set_lights(&state).await?;
                    println!("Scene applied: brightness {}%, temperature {}K", brightness, kelvin);
                }
                
                _ => unreachable!(),
            }
        }
    }
    
    Ok(())
}