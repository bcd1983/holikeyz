use anyhow::Result;
use clap::{Parser, Subcommand};
use holikeyz::provisioning::elgato_fixed::ElgatoDirectProvisioner;
use log::info;
use env_logger;

#[derive(Parser)]
#[clap(name = "elgato-debug")]
#[clap(about = "Debug tool for Elgato Ring Light provisioning")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    
    /// Device IP address
    #[clap(short, long, default_value = "192.168.62.1")]
    ip: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Check available endpoints
    Check,
    
    /// Get device information
    Info,
    
    /// Get WiFi information
    Wifi,
    
    /// Provision device
    Provision {
        /// WiFi SSID
        ssid: String,
        
        /// WiFi password
        #[clap(short, long)]
        password: Option<String>,
    },
    
    /// Test all provisioning methods
    TestAll {
        /// WiFi SSID
        ssid: String,
        
        /// WiFi password
        password: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    
    let cli = Cli::parse();
    let provisioner = ElgatoDirectProvisioner::with_ip(&cli.ip);
    
    match cli.command {
        Commands::Check => {
            println!("Checking device at {}...", cli.ip);
            provisioner.check_endpoints().await?;
        }
        
        Commands::Info => {
            println!("Getting device info from {}...", cli.ip);
            let info = provisioner.get_device_info().await?;
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        
        Commands::Wifi => {
            println!("Getting WiFi info from {}...", cli.ip);
            match provisioner.get_wifi_info().await {
                Ok(info) => println!("{}", serde_json::to_string_pretty(&info)?),
                Err(e) => println!("Error: {}", e),
            }
        }
        
        Commands::Provision { ssid, password } => {
            let password = password.unwrap_or_else(|| {
                rpassword::prompt_password("Enter WiFi password: ").unwrap()
            });
            
            println!("Provisioning device for network: {}", ssid);
            provisioner.provision_device(&ssid, &password).await?;
            println!("✅ Provisioning complete!");
        }
        
        Commands::TestAll { ssid, password } => {
            println!("Testing all provisioning methods for network: {}", ssid);
            println!("{}", "=".repeat(50));
            
            // Test 1: Simple JSON
            println!("\n1. Testing simple JSON method...");
            match provisioner.set_wifi_simple(&ssid, &password).await {
                Ok(_) => println!("   ✅ Success!"),
                Err(e) => println!("   ❌ Failed: {}", e),
            }
            
            // Test 2: Standard JSON with wrapper
            println!("\n2. Testing standard JSON method...");
            match provisioner.set_wifi_json(&ssid, &password).await {
                Ok(_) => println!("   ✅ Success!"),
                Err(e) => println!("   ❌ Failed: {}", e),
            }
            
            // Test 3: Direct JSON
            println!("\n3. Testing direct JSON method...");
            let client = reqwest::Client::new();
            let url = format!("http://{}:9123/elgato/wifi-info", cli.ip);
            let config = serde_json::json!({
                "ssid": ssid,
                "passphrase": password,
                "priority": 1
            });
            
            match client.put(&url).json(&config).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("   ✅ Success!");
                    } else {
                        let status = response.status();
                        let text = response.text().await.unwrap_or_default();
                        println!("   ❌ Failed: {} - {}", status, text);
                    }
                }
                Err(e) => println!("   ❌ Failed: {}", e),
            }
            
            // Test 4: Form encoded
            println!("\n4. Testing form-encoded method...");
            let params = [
                ("ssid", ssid.as_str()),
                ("passphrase", password.as_str()),
            ];
            
            match client.put(&url).form(&params).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("   ✅ Success!");
                    } else {
                        let status = response.status();
                        let text = response.text().await.unwrap_or_default();
                        println!("   ❌ Failed: {} - {}", status, text);
                    }
                }
                Err(e) => println!("   ❌ Failed: {}", e),
            }
            
            println!("\n{}", "=".repeat(50));
        }
    }
    
    Ok(())
}