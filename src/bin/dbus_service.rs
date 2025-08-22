use anyhow::Result;
use dbus::blocking::Connection;
use dbus_crossroads::{Crossroads, IfaceBuilder};
use elgato_controller::{ElgatoClient, models::LightState, error::ElgatoError};
use std::sync::{Arc, Mutex, RwLock};
use log::info;
use tokio::runtime::Runtime;

const DBUS_NAME: &str = "com.elgato.RingLight";
const DBUS_PATH: &str = "/com/elgato/RingLight";
const DBUS_INTERFACE: &str = "com.elgato.RingLight.Control";

struct CachedState {
    is_on: bool,
    brightness: u8,
    temperature: u16,
}

struct LightService {
    client: Arc<ElgatoClient>,
    runtime: Arc<Runtime>,
    cached_state: Arc<RwLock<CachedState>>,
}

impl LightService {
    fn new(ip: &str, port: u16) -> Self {
        let client = ElgatoClient::new(ip, port);
        let runtime = Runtime::new().unwrap();
        
        // Initialize with default state
        let cached_state = CachedState {
            is_on: false,
            brightness: 50,
            temperature: 213,
        };
        
        let service = Self {
            client: Arc::new(client),
            runtime: Arc::new(runtime),
            cached_state: Arc::new(RwLock::new(cached_state)),
        };
        
        // Fetch initial state asynchronously
        service.update_cached_state();
        
        service
    }
    
    fn update_cached_state(&self) {
        let client = self.client.clone();
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        std::thread::spawn(move || {
            runtime.block_on(async {
                if let Ok(response) = client.get_lights().await {
                    if let Some(light) = response.lights.first() {
                        let mut cache = cached_state.write().unwrap();
                        cache.is_on = light.is_on();
                        cache.brightness = light.brightness;
                        cache.temperature = light.temperature;
                    }
                }
            });
        });
    }
    
    fn turn_on(&self) -> Result<bool> {
        let client = self.client.clone();
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        // Update cache immediately for responsiveness
        {
            let mut cache = cached_state.write().unwrap();
            cache.is_on = true;
        }
        
        // Send command asynchronously
        std::thread::spawn(move || {
            runtime.block_on(async {
                let _ = client.turn_on().await;
            });
        });
        
        Ok(true)
    }
    
    fn turn_off(&self) -> Result<bool> {
        let client = self.client.clone();
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        // Update cache immediately
        {
            let mut cache = cached_state.write().unwrap();
            cache.is_on = false;
        }
        
        // Send command asynchronously
        std::thread::spawn(move || {
            runtime.block_on(async {
                let _ = client.turn_off().await;
            });
        });
        
        Ok(true)
    }
    
    fn toggle(&self) -> Result<bool> {
        let is_on = {
            let cache = self.cached_state.read().unwrap();
            cache.is_on
        };
        
        if is_on {
            self.turn_off()
        } else {
            self.turn_on()
        }
    }
    
    fn set_brightness(&self, brightness: u8) -> Result<bool> {
        if brightness > 100 {
            return Err(anyhow::anyhow!("Brightness must be 0-100"));
        }
        
        let client = self.client.clone();
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        // Get current state and update brightness
        let (is_on, temperature) = {
            let mut cache = cached_state.write().unwrap();
            cache.brightness = brightness;
            cache.is_on = true;  // Turn on when adjusting
            (cache.is_on, cache.temperature)
        };
        
        // Send command asynchronously with current temperature
        std::thread::spawn(move || {
            runtime.block_on(async {
                let mut state = LightState::new();
                state.set_on(true);
                state.brightness = brightness;
                state.temperature = temperature;
                let _ = client.set_lights(&state).await;
            });
        });
        
        Ok(true)
    }
    
    fn set_temperature(&self, kelvin: u32) -> Result<bool> {
        if !(2900..=7000).contains(&kelvin) {
            return Err(anyhow::anyhow!("Temperature must be 2900-7000K"));
        }
        
        let client = self.client.clone();
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        let api_temp = LightState::kelvin_to_api(kelvin);
        
        // Get current state and update temperature
        let (is_on, brightness) = {
            let mut cache = cached_state.write().unwrap();
            cache.temperature = api_temp;
            cache.is_on = true;  // Turn on when adjusting
            (cache.is_on, cache.brightness)
        };
        
        // Send command asynchronously with current brightness
        std::thread::spawn(move || {
            runtime.block_on(async {
                let mut state = LightState::new();
                state.set_on(true);
                state.brightness = brightness;
                state.temperature = api_temp;
                let _ = client.set_lights(&state).await;
            });
        });
        
        Ok(true)
    }
    
    fn get_status(&self) -> Result<(bool, u8, u32)> {
        // Return cached state immediately
        let cache = self.cached_state.read().unwrap();
        Ok((
            cache.is_on,
            cache.brightness,
            LightState::api_to_kelvin(cache.temperature)
        ))
    }
    
    fn identify(&self) -> Result<bool> {
        let client = self.client.clone();
        let runtime = self.runtime.clone();
        
        // Fire and forget
        std::thread::spawn(move || {
            runtime.block_on(async {
                let _ = client.identify().await;
            });
        });
        
        Ok(true)
    }
    
    fn apply_scene(&self, scene: &str) -> Result<bool> {
        let (brightness, kelvin) = match scene {
            "daylight" => (80, 5600),
            "warm" => (60, 3200),
            "cool" => (70, 6500),
            "reading" => (90, 4500),
            "video" => (75, 5000),
            "relax" => (40, 2900),
            _ => return Err(anyhow::anyhow!("Unknown scene: {}", scene)),
        };
        
        let client = self.client.clone();
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        let api_temp = LightState::kelvin_to_api(kelvin);
        
        // Update cache immediately
        {
            let mut cache = cached_state.write().unwrap();
            cache.is_on = true;
            cache.brightness = brightness;
            cache.temperature = api_temp;
        }
        
        // Send command asynchronously
        std::thread::spawn(move || {
            runtime.block_on(async {
                let mut state = LightState::new();
                state.set_on(true);
                state.brightness = brightness;
                state.temperature = api_temp;
                let _ = client.set_lights(&state).await;
            });
        });
        
        Ok(true)
    }
}

fn register_interface(cr: &mut Crossroads, service: Arc<LightService>) -> dbus_crossroads::IfaceToken<Arc<LightService>> {
    cr.register(DBUS_INTERFACE, |b: &mut IfaceBuilder<Arc<LightService>>| {
        b.method("TurnOn", (), ("success",), |_, service, _: ()| {
            service.turn_on()
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("TurnOff", (), ("success",), |_, service, _: ()| {
            service.turn_off()
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("Toggle", (), ("success",), |_, service, _: ()| {
            service.toggle()
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("SetBrightness", ("brightness",), ("success",), |_, service, (brightness,): (u8,)| {
            service.set_brightness(brightness)
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("SetTemperature", ("kelvin",), ("success",), |_, service, (kelvin,): (u32,)| {
            service.set_temperature(kelvin)
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("GetStatus", (), ("is_on", "brightness", "temperature"), |_, service, _: ()| {
            service.get_status()
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("Identify", (), ("success",), |_, service, _: ()| {
            service.identify()
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("ApplyScene", ("scene",), ("success",), |_, service, (scene,): (String,)| {
            service.apply_scene(&scene)
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.property("IsOn")
            .get(|_, service| {
                service.get_status()
                    .map(|(is_on, _, _)| is_on)
                    .map_err(|e| dbus::MethodErr::failed(&e))
            });
        
        b.property("Brightness")
            .get(|_, service| {
                service.get_status()
                    .map(|(_, brightness, _)| brightness)
                    .map_err(|e| dbus::MethodErr::failed(&e))
            })
            .set(|_, service, brightness| {
                service.set_brightness(brightness)
                    .map(|_| Some(brightness))
                    .map_err(|e| dbus::MethodErr::failed(&e))
            });
        
        b.property("Temperature")
            .get(|_, service| {
                service.get_status()
                    .map(|(_, _, temperature)| temperature)
                    .map_err(|e| dbus::MethodErr::failed(&e))
            })
            .set(|_, service, kelvin| {
                service.set_temperature(kelvin)
                    .map(|_| Some(kelvin))
                    .map_err(|e| dbus::MethodErr::failed(&e))
            });
    })
}

fn main() -> Result<()> {
    env_logger::init();
    
    let ip = std::env::var("ELGATO_IP").unwrap_or_else(|_| "192.168.7.80".to_string());
    let port = std::env::var("ELGATO_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(9123);
    
    info!("Starting optimized Elgato Ring Light D-Bus service");
    info!("Connecting to light at {}:{}", ip, port);
    
    let service = Arc::new(LightService::new(&ip, port));
    
    let conn = Connection::new_session()?;
    conn.request_name(DBUS_NAME, false, true, false)?;
    
    let mut cr = Crossroads::new();
    let token = register_interface(&mut cr, service.clone());
    cr.insert(DBUS_PATH, &[token], service);
    
    info!("D-Bus service registered as {}", DBUS_NAME);
    info!("Listening on path {}", DBUS_PATH);
    
    cr.serve(&conn)?;
    unreachable!()
}