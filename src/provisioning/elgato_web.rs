use anyhow::{Result, Context};
use log::{info, debug, warn};
use std::process::Command;

/// Alternative provisioning method using the device's web interface
/// This opens the browser to the device's configuration page
pub struct ElgatoWebProvisioner;

impl ElgatoWebProvisioner {
    pub fn new() -> Self {
        Self
    }
    
    /// Open the device's web interface for manual provisioning
    pub fn open_web_interface(&self, device_ip: &str) -> Result<()> {
        let url = format!("http://{}:9123", device_ip);
        info!("Opening device web interface at: {}", url);
        
        // Try different commands to open the browser
        let commands = vec![
            ("xdg-open", vec![&url]),
            ("open", vec![&url]),  // macOS
            ("firefox", vec![&url]),
            ("chromium", vec![&url]),
            ("google-chrome", vec![&url]),
        ];
        
        for (cmd, args) in commands {
            if Command::new(cmd).args(&args).spawn().is_ok() {
                info!("Opened browser with {}", cmd);
                return Ok(());
            }
        }
        
        // If no browser could be opened, show the URL
        warn!("Could not open browser automatically");
        println!("\n📱 Manual Configuration Required");
        println!("================================");
        println!("Please open your browser and navigate to:");
        println!("  {}", url);
        println!("\nOn the web interface:");
        println!("1. Click on 'WiFi Settings'");
        println!("2. Select your network from the list");
        println!("3. Enter your WiFi password");
        println!("4. Click 'Connect'");
        println!("\nThe device will reboot and connect to your network.");
        
        Ok(())
    }
    
    /// Show instructions for manual provisioning
    pub fn show_manual_instructions(&self) -> Result<()> {
        println!("\n📋 Manual Provisioning Instructions");
        println!("====================================");
        println!();
        println!("If automatic provisioning fails, you can configure the device manually:");
        println!();
        println!("Option 1: Web Interface");
        println!("-----------------------");
        println!("1. Connect to the device's WiFi network (Elgato Ring Light XXXX)");
        println!("2. Open a browser and go to: http://192.168.62.1:9123");
        println!("3. Click 'WiFi Settings' and select your network");
        println!("4. Enter your password and click 'Connect'");
        println!();
        println!("Option 2: Elgato Control Center App");
        println!("------------------------------------");
        println!("1. Download 'Elgato Control Center' from the App Store or Google Play");
        println!("2. Open the app and tap 'Add Device'");
        println!("3. Follow the in-app instructions");
        println!();
        println!("Option 3: Using This Tool");
        println!("-------------------------");
        println!("Run: elgato-enhanced setup --use-saved");
        println!("This will prompt for credentials if not saved");
        
        Ok(())
    }
}

/// Check if we can reach the device's web interface
pub async fn check_web_interface(device_ip: &str) -> Result<bool> {
    let url = format!("http://{}:9123", device_ip);
    
    match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?
        .get(&url)
        .send()
        .await
    {
        Ok(response) => {
            debug!("Web interface reachable: {} - {}", url, response.status());
            Ok(response.status().is_success() || response.status().as_u16() == 401)
        }
        Err(e) => {
            debug!("Web interface not reachable: {} - {}", url, e);
            Ok(false)
        }
    }
}

/// Fallback provisioning using web interface
pub async fn provision_via_web(device_ip: &str, ssid: &str) -> Result<()> {
    info!("Attempting web-based provisioning for network: {}", ssid);
    
    // Check if web interface is available
    if !check_web_interface(device_ip).await? {
        return Err(anyhow::anyhow!("Device web interface not reachable at {}", device_ip));
    }
    
    let provisioner = ElgatoWebProvisioner::new();
    provisioner.open_web_interface(device_ip)?;
    
    println!("\n⏳ Waiting for you to complete setup in the browser...");
    println!("   Network to select: {}", ssid);
    println!("   Press Enter when done...");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)
        .context("Failed to read input")?;
    
    Ok(())
}