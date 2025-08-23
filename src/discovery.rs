use crate::{error::Result, models::LightInfo, HolikeyzError};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::collections::HashMap;
use std::time::Duration;
use log::{debug, info};

// Using Elgato's mDNS service type for discovery (device compatibility)
const RING_LIGHT_SERVICE: &str = "_elg._tcp.local.";

pub async fn discover_lights(timeout: Duration) -> Result<Vec<LightInfo>> {
    let mdns = ServiceDaemon::new()
        .map_err(|e| HolikeyzError::DiscoveryError(e.to_string()))?;
    
    let receiver = mdns.browse(RING_LIGHT_SERVICE)
        .map_err(|e| HolikeyzError::DiscoveryError(e.to_string()))?;
    
    let mut lights = HashMap::new();
    let start = std::time::Instant::now();
    
    while start.elapsed() < timeout {
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(ServiceEvent::ServiceResolved(info)) => {
                debug!("Found service: {:?}", info);
                
                if let Some(addr) = info.get_addresses().iter().next() {
                    let light_info = LightInfo {
                        ip: addr.to_string(),
                        port: info.get_port(),
                        name: info.get_fullname().to_string(),
                        state: None,
                        accessory_info: None,
                    };
                    
                    info!("Discovered Ring Light: {} at {}:{}", 
                          light_info.name, light_info.ip, light_info.port);
                    
                    lights.insert(info.get_fullname().to_string(), light_info);
                }
            }
            Ok(ServiceEvent::ServiceRemoved(_, fullname)) => {
                debug!("Service removed: {}", fullname);
                lights.remove(&fullname);
            }
            Ok(_) => {}
            Err(_) => {}
        }
    }
    
    Ok(lights.into_values().collect())
}

pub async fn find_light_by_ip(ip: &str, port: u16) -> Result<LightInfo> {
    Ok(LightInfo {
        ip: ip.to_string(),
        port,
        name: format!("ring-light-{}", ip),
        state: None,
        accessory_info: None,
    })
}