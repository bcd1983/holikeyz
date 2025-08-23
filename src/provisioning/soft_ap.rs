use anyhow::{Result, Context};
use std::net::{IpAddr, Ipv4Addr};
use std::process::Command;
use log::{info, debug};
use tokio::time::{sleep, Duration};

pub struct SoftAPConfig {
    pub interface: String,
    pub ssid: String,
    pub password: Option<String>,
    pub ip_address: Ipv4Addr,
    pub netmask: Ipv4Addr,
    pub channel: u8,
    pub dhcp_range_start: Ipv4Addr,
    pub dhcp_range_end: Ipv4Addr,
}

impl Default for SoftAPConfig {
    fn default() -> Self {
        Self {
            interface: "wlan0".to_string(),
            ssid: "Holikeyz-Setup".to_string(),
            password: None,
            ip_address: Ipv4Addr::new(192, 168, 4, 1),
            netmask: Ipv4Addr::new(255, 255, 255, 0),
            channel: 6,
            dhcp_range_start: Ipv4Addr::new(192, 168, 4, 100),
            dhcp_range_end: Ipv4Addr::new(192, 168, 4, 200),
        }
    }
}

pub struct SoftAPManager {
    config: SoftAPConfig,
    hostapd_running: bool,
    dnsmasq_running: bool,
}

impl SoftAPManager {
    pub fn new(config: SoftAPConfig) -> Self {
        Self {
            config,
            hostapd_running: false,
            dnsmasq_running: false,
        }
    }

    pub async fn start(&mut self) -> Result<IpAddr> {
        info!("Starting Soft AP with SSID: {}", self.config.ssid);
        
        self.stop_network_manager().await?;
        self.configure_interface().await?;
        self.start_hostapd().await?;
        self.start_dnsmasq().await?;
        self.configure_iptables().await?;
        
        Ok(IpAddr::V4(self.config.ip_address))
    }

    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping Soft AP");
        
        if self.hostapd_running {
            self.stop_hostapd().await?;
        }
        
        if self.dnsmasq_running {
            self.stop_dnsmasq().await?;
        }
        
        self.restore_network_manager().await?;
        self.cleanup_iptables().await?;
        
        Ok(())
    }

    async fn stop_network_manager(&self) -> Result<()> {
        debug!("Stopping NetworkManager for interface {}", self.config.interface);
        
        Command::new("nmcli")
            .args(&["device", "set", &self.config.interface, "managed", "no"])
            .output()
            .context("Failed to unmanage interface with NetworkManager")?;
        
        sleep(Duration::from_secs(1)).await;
        Ok(())
    }

    async fn configure_interface(&self) -> Result<()> {
        debug!("Configuring interface {}", self.config.interface);
        
        Command::new("ip")
            .args(&["link", "set", &self.config.interface, "down"])
            .output()
            .context("Failed to bring interface down")?;
        
        Command::new("ip")
            .args(&["addr", "flush", "dev", &self.config.interface])
            .output()
            .context("Failed to flush interface addresses")?;
        
        Command::new("ip")
            .args(&[
                "addr", "add",
                &format!("{}/{}", self.config.ip_address, "24"),
                "dev", &self.config.interface
            ])
            .output()
            .context("Failed to set interface IP address")?;
        
        Command::new("ip")
            .args(&["link", "set", &self.config.interface, "up"])
            .output()
            .context("Failed to bring interface up")?;
        
        Ok(())
    }

    async fn start_hostapd(&mut self) -> Result<()> {
        debug!("Starting hostapd");
        
        let hostapd_conf = self.generate_hostapd_config()?;
        std::fs::write("/tmp/hostapd.conf", hostapd_conf)
            .context("Failed to write hostapd config")?;
        
        Command::new("hostapd")
            .args(&["-B", "/tmp/hostapd.conf"])
            .spawn()
            .context("Failed to start hostapd")?;
        
        self.hostapd_running = true;
        sleep(Duration::from_secs(2)).await;
        
        Ok(())
    }

    async fn start_dnsmasq(&mut self) -> Result<()> {
        debug!("Starting dnsmasq");
        
        let dnsmasq_conf = self.generate_dnsmasq_config();
        std::fs::write("/tmp/dnsmasq.conf", dnsmasq_conf)
            .context("Failed to write dnsmasq config")?;
        
        Command::new("dnsmasq")
            .args(&["-C", "/tmp/dnsmasq.conf"])
            .spawn()
            .context("Failed to start dnsmasq")?;
        
        self.dnsmasq_running = true;
        sleep(Duration::from_secs(1)).await;
        
        Ok(())
    }

    async fn configure_iptables(&self) -> Result<()> {
        debug!("Configuring iptables for NAT");
        
        Command::new("sysctl")
            .args(&["-w", "net.ipv4.ip_forward=1"])
            .output()
            .context("Failed to enable IP forwarding")?;
        
        Command::new("iptables")
            .args(&["-t", "nat", "-A", "POSTROUTING", "-o", "eth0", "-j", "MASQUERADE"])
            .output()
            .context("Failed to configure NAT")?;
        
        Command::new("iptables")
            .args(&["-A", "FORWARD", "-i", &self.config.interface, "-o", "eth0", "-j", "ACCEPT"])
            .output()
            .context("Failed to configure forwarding")?;
        
        Command::new("iptables")
            .args(&["-A", "FORWARD", "-i", "eth0", "-o", &self.config.interface, "-m", "state", "--state", "RELATED,ESTABLISHED", "-j", "ACCEPT"])
            .output()
            .context("Failed to configure return traffic")?;
        
        Ok(())
    }

    async fn stop_hostapd(&mut self) -> Result<()> {
        debug!("Stopping hostapd");
        
        Command::new("pkill")
            .arg("hostapd")
            .output()
            .context("Failed to stop hostapd")?;
        
        self.hostapd_running = false;
        Ok(())
    }

    async fn stop_dnsmasq(&mut self) -> Result<()> {
        debug!("Stopping dnsmasq");
        
        Command::new("pkill")
            .arg("dnsmasq")
            .output()
            .context("Failed to stop dnsmasq")?;
        
        self.dnsmasq_running = false;
        Ok(())
    }

    async fn restore_network_manager(&self) -> Result<()> {
        debug!("Restoring NetworkManager for interface {}", self.config.interface);
        
        Command::new("nmcli")
            .args(&["device", "set", &self.config.interface, "managed", "yes"])
            .output()
            .context("Failed to manage interface with NetworkManager")?;
        
        Ok(())
    }

    async fn cleanup_iptables(&self) -> Result<()> {
        debug!("Cleaning up iptables rules");
        
        let _ = Command::new("iptables")
            .args(&["-t", "nat", "-D", "POSTROUTING", "-o", "eth0", "-j", "MASQUERADE"])
            .output();
        
        let _ = Command::new("iptables")
            .args(&["-D", "FORWARD", "-i", &self.config.interface, "-o", "eth0", "-j", "ACCEPT"])
            .output();
        
        let _ = Command::new("iptables")
            .args(&["-D", "FORWARD", "-i", "eth0", "-o", &self.config.interface, "-m", "state", "--state", "RELATED,ESTABLISHED", "-j", "ACCEPT"])
            .output();
        
        Ok(())
    }

    fn generate_hostapd_config(&self) -> Result<String> {
        let mut config = format!(
            r#"interface={}
driver=nl80211
ssid={}
hw_mode=g
channel={}
macaddr_acl=0
auth_algs=1
ignore_broadcast_ssid=0
"#,
            self.config.interface,
            self.config.ssid,
            self.config.channel
        );

        if let Some(password) = &self.config.password {
            if password.len() < 8 {
                anyhow::bail!("WiFi password must be at least 8 characters");
            }
            config.push_str(&format!(
                r#"wpa=2
wpa_passphrase={}
wpa_key_mgmt=WPA-PSK
wpa_pairwise=TKIP
rsn_pairwise=CCMP
"#,
                password
            ));
        }

        Ok(config)
    }

    fn generate_dnsmasq_config(&self) -> String {
        format!(
            r#"interface={}
bind-interfaces
dhcp-range={},{},12h
dhcp-option=3,{}
dhcp-option=6,{}
server=8.8.8.8
server=8.8.4.4
log-queries
log-dhcp
"#,
            self.config.interface,
            self.config.dhcp_range_start,
            self.config.dhcp_range_end,
            self.config.ip_address,
            self.config.ip_address
        )
    }
}

pub async fn is_soft_ap_capable() -> Result<bool> {
    let output = Command::new("iw")
        .args(&["list"])
        .output()
        .context("Failed to check wireless capabilities")?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.contains("AP"))
}