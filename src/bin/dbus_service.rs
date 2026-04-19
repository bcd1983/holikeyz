use anyhow::Result;
use dbus::blocking::Connection;
use dbus_crossroads::{Crossroads, IfaceBuilder};
use holikeyz::{RingLightClient, ActiveLight, active_config_path, load_active, resolve_active, save_active, models::{LightState, AccessoryInfo, Settings}};
use serde::Serialize;
use std::sync::{Arc, RwLock};
use log::{info, warn};
use tokio::runtime::Runtime;

const DBUS_NAME: &str = "com.holikeyz.RingLight";
const DBUS_PATH: &str = "/com/holikeyz/RingLight";
const DBUS_INTERFACE: &str = "com.holikeyz.RingLight.Control";

#[derive(Debug, Clone, Serialize)]
struct DiscoveredLight {
    name: String,
    ip: String,
    port: u16,
}

struct CachedState {
    is_on: Vec<bool>,
    brightness: Vec<u8>,
    temperature: Vec<u16>,
    accessory_info: Option<AccessoryInfo>,
    settings: Option<Settings>,
    num_lights: u8,
}

struct LightService {
    client: Arc<RwLock<Option<Arc<RingLightClient>>>>,
    active: Arc<RwLock<Option<ActiveLight>>>,
    runtime: Arc<Runtime>,
    cached_state: Arc<RwLock<CachedState>>,
}

impl LightService {
    fn new(initial: Option<ActiveLight>) -> Self {
        let runtime = Runtime::new().unwrap();
        let cached_state = CachedState {
            is_on: vec![false],
            brightness: vec![50],
            temperature: vec![213],
            accessory_info: None,
            settings: None,
            num_lights: 1,
        };

        let (client, active) = match initial {
            Some(a) => {
                let c = Arc::new(RingLightClient::new(&a.ip, a.port));
                (Some(c), Some(a))
            }
            None => (None, None),
        };

        let service = Self {
            client: Arc::new(RwLock::new(client)),
            active: Arc::new(RwLock::new(active)),
            runtime: Arc::new(runtime),
            cached_state: Arc::new(RwLock::new(cached_state)),
        };

        // Fetch initial state asynchronously (no-op if no active light)
        service.update_cached_state();

        service
    }

    fn client(&self) -> Result<Arc<RingLightClient>> {
        self.client
            .read()
            .unwrap()
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("no active light — call Discover then SetActiveLight"))
    }

    fn get_active_light(&self) -> (String, u16) {
        match self.active.read().unwrap().as_ref() {
            Some(a) => (a.ip.clone(), a.port),
            None => (String::new(), 0),
        }
    }

    fn set_active_light(&self, ip: String, port: u16) -> Result<bool> {
        if ip.trim().is_empty() {
            anyhow::bail!("IP must not be empty");
        }
        let new_client = Arc::new(RingLightClient::new(&ip, port));
        let new_active = ActiveLight { ip: ip.clone(), port };

        *self.client.write().unwrap() = Some(new_client);
        *self.active.write().unwrap() = Some(new_active.clone());

        if let Err(e) = save_active(&new_active) {
            warn!("failed to persist active light: {}", e);
        } else {
            info!("active light switched to {}:{}", ip, port);
        }

        self.update_cached_state();
        Ok(true)
    }

    fn discover(&self, timeout_secs: u32) -> Result<String> {
        use std::time::Duration;
        let timeout = Duration::from_secs(timeout_secs.max(1).min(30) as u64);
        let runtime = self.runtime.clone();

        let lights = std::thread::spawn(move || -> Vec<holikeyz::LightInfo> {
            runtime.block_on(async move {
                holikeyz::discover_lights(timeout).await.unwrap_or_default()
            })
        })
        .join()
        .map_err(|_| anyhow::anyhow!("discovery thread panicked"))?;

        let simplified: Vec<DiscoveredLight> = lights
            .into_iter()
            .map(|l| DiscoveredLight { name: l.name, ip: l.ip, port: l.port })
            .collect();
        Ok(serde_json::to_string(&simplified)?)
    }
    
    fn update_cached_state(&self) {
        // No-op when there's no active light yet; the cache stays at defaults
        // until SetActiveLight is called.
        let client = match self.client() {
            Ok(c) => c,
            Err(_) => return,
        };
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        std::thread::spawn(move || {
            runtime.block_on(async {
                if let Ok(response) = client.get_lights().await {
                    let mut cache = cached_state.write().unwrap();
                    cache.num_lights = response.number_of_lights;
                    cache.is_on.clear();
                    cache.brightness.clear();
                    cache.temperature.clear();
                    
                    for light in &response.lights {
                        cache.is_on.push(light.is_on());
                        cache.brightness.push(light.brightness);
                        cache.temperature.push(light.temperature);
                    }
                }
                
                // Also fetch accessory info and settings
                if let Ok(info) = client.get_accessory_info().await {
                    let mut cache = cached_state.write().unwrap();
                    cache.accessory_info = Some(info);
                }
                
                if let Ok(settings) = client.get_settings().await {
                    let mut cache = cached_state.write().unwrap();
                    cache.settings = Some(settings);
                }
            });
        });
    }
    
    fn turn_on(&self) -> Result<bool> {
        self.turn_on_light(None)
    }
    
    fn turn_on_light(&self, light_index: Option<u8>) -> Result<bool> {
        let client = self.client()?;
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        // Update cache immediately for responsiveness
        {
            let mut cache = cached_state.write().unwrap();
            if let Some(idx) = light_index {
                if (idx as usize) < cache.is_on.len() {
                    cache.is_on[idx as usize] = true;
                }
            } else {
                // Turn on all lights
                for state in &mut cache.is_on {
                    *state = true;
                }
            }
        }
        
        // Send command asynchronously
        std::thread::spawn(move || {
            runtime.block_on(async {
                if let Some(idx) = light_index {
                    // Turn on specific light
                    if let Ok(mut response) = client.get_lights().await {
                        if let Some(light) = response.lights.get_mut(idx as usize) {
                            light.set_on(true);
                            let _ = client.set_lights(light).await;
                        }
                    }
                } else {
                    // Turn on all lights
                    let _ = client.turn_on().await;
                }
            });
        });
        
        Ok(true)
    }
    
    fn turn_off(&self) -> Result<bool> {
        self.turn_off_light(None)
    }
    
    fn turn_off_light(&self, light_index: Option<u8>) -> Result<bool> {
        let client = self.client()?;
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        // Update cache immediately
        {
            let mut cache = cached_state.write().unwrap();
            if let Some(idx) = light_index {
                if (idx as usize) < cache.is_on.len() {
                    cache.is_on[idx as usize] = false;
                }
            } else {
                // Turn off all lights
                for state in &mut cache.is_on {
                    *state = false;
                }
            }
        }
        
        // Send command asynchronously
        std::thread::spawn(move || {
            runtime.block_on(async {
                if let Some(idx) = light_index {
                    // Turn off specific light
                    if let Ok(mut response) = client.get_lights().await {
                        if let Some(light) = response.lights.get_mut(idx as usize) {
                            light.set_on(false);
                            let _ = client.set_lights(light).await;
                        }
                    }
                } else {
                    // Turn off all lights
                    let _ = client.turn_off().await;
                }
            });
        });
        
        Ok(true)
    }
    
    fn toggle(&self) -> Result<bool> {
        let is_on = {
            let cache = self.cached_state.read().unwrap();
            // Check if any light is on
            cache.is_on.iter().any(|&on| on)
        };
        
        if is_on {
            self.turn_off()
        } else {
            self.turn_on()
        }
    }
    
    fn toggle_light(&self, light_index: u8) -> Result<bool> {
        let is_on = {
            let cache = self.cached_state.read().unwrap();
            if (light_index as usize) < cache.is_on.len() {
                cache.is_on[light_index as usize]
            } else {
                false
            }
        };
        
        if is_on {
            self.turn_off_light(Some(light_index))
        } else {
            self.turn_on_light(Some(light_index))
        }
    }
    
    fn set_brightness(&self, brightness: u8) -> Result<bool> {
        self.set_brightness_light(brightness, None)
    }
    
    fn set_brightness_light(&self, brightness: u8, light_index: Option<u8>) -> Result<bool> {
        if brightness > 100 {
            return Err(anyhow::anyhow!("Brightness must be 0-100"));
        }
        
        let client = self.client()?;
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        // Get current state and update brightness
        let temperatures = {
            let mut cache = cached_state.write().unwrap();
            if let Some(idx) = light_index {
                if (idx as usize) < cache.brightness.len() {
                    cache.brightness[idx as usize] = brightness;
                    cache.is_on[idx as usize] = true;
                }
            } else {
                // Update all lights
                for i in 0..cache.brightness.len() {
                    cache.brightness[i] = brightness;
                    cache.is_on[i] = true;
                }
            }
            cache.temperature.clone()
        };
        
        // Send command asynchronously with current temperature
        let light_idx = light_index;
        std::thread::spawn(move || {
            runtime.block_on(async {
                if let Some(idx) = light_idx {
                    if (idx as usize) < temperatures.len() {
                        let mut state = LightState::new();
                        state.set_on(true);
                        state.brightness = brightness;
                        state.temperature = temperatures[idx as usize];
                        let _ = client.set_lights(&state).await;
                    }
                } else {
                    // Set all lights
                    let mut state = LightState::new();
                    state.set_on(true);
                    state.brightness = brightness;
                    state.temperature = temperatures.first().copied().unwrap_or(213);
                    let _ = client.set_lights(&state).await;
                }
            });
        });
        
        Ok(true)
    }
    
    fn set_temperature(&self, kelvin: u32) -> Result<bool> {
        self.set_temperature_light(kelvin, None)
    }
    
    fn set_temperature_light(&self, kelvin: u32, light_index: Option<u8>) -> Result<bool> {
        if !(2900..=7000).contains(&kelvin) {
            return Err(anyhow::anyhow!("Temperature must be 2900-7000K"));
        }
        
        let client = self.client()?;
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        let api_temp = LightState::kelvin_to_api(kelvin);
        
        // Get current state and update temperature
        let brightnesses = {
            let mut cache = cached_state.write().unwrap();
            if let Some(idx) = light_index {
                if (idx as usize) < cache.temperature.len() {
                    cache.temperature[idx as usize] = api_temp;
                    cache.is_on[idx as usize] = true;
                }
            } else {
                // Update all lights
                for i in 0..cache.temperature.len() {
                    cache.temperature[i] = api_temp;
                    cache.is_on[i] = true;
                }
            }
            cache.brightness.clone()
        };
        
        // Send command asynchronously with current brightness
        let light_idx = light_index;
        std::thread::spawn(move || {
            runtime.block_on(async {
                if let Some(idx) = light_idx {
                    if (idx as usize) < brightnesses.len() {
                        let mut state = LightState::new();
                        state.set_on(true);
                        state.brightness = brightnesses[idx as usize];
                        state.temperature = api_temp;
                        let _ = client.set_lights(&state).await;
                    }
                } else {
                    // Set all lights
                    let mut state = LightState::new();
                    state.set_on(true);
                    state.brightness = brightnesses.first().copied().unwrap_or(50);
                    state.temperature = api_temp;
                    let _ = client.set_lights(&state).await;
                }
            });
        });
        
        Ok(true)
    }
    
    fn get_status(&self) -> Result<(bool, u8, u32)> {
        // Return cached state immediately (first light for backward compatibility)
        let cache = self.cached_state.read().unwrap();
        Ok((
            cache.is_on.first().copied().unwrap_or(false),
            cache.brightness.first().copied().unwrap_or(50),
            LightState::api_to_kelvin(cache.temperature.first().copied().unwrap_or(213))
        ))
    }
    
    fn get_num_lights(&self) -> Result<u8> {
        let cache = self.cached_state.read().unwrap();
        Ok(cache.num_lights)
    }
    
    fn get_all_lights_status(&self) -> Result<(Vec<bool>, Vec<u8>, Vec<u32>)> {
        let cache = self.cached_state.read().unwrap();
        let temps_kelvin: Vec<u32> = cache.temperature.iter()
            .map(|&t| LightState::api_to_kelvin(t))
            .collect();
        Ok((
            cache.is_on.clone(),
            cache.brightness.clone(),
            temps_kelvin
        ))
    }
    
    fn get_light_status(&self, light_index: u8) -> Result<(bool, u8, u32)> {
        let cache = self.cached_state.read().unwrap();
        let idx = light_index as usize;
        
        if idx >= cache.num_lights as usize {
            return Err(anyhow::anyhow!("Light index {} out of range", light_index));
        }
        
        Ok((
            cache.is_on.get(idx).copied().unwrap_or(false),
            cache.brightness.get(idx).copied().unwrap_or(50),
            LightState::api_to_kelvin(cache.temperature.get(idx).copied().unwrap_or(213))
        ))
    }
    
    fn get_accessory_info(&self) -> Result<(String, String, String, u32, String, Vec<String>)> {
        let client = self.client()?;
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        // Try to get from cache first
        {
            let cache = cached_state.read().unwrap();
            if let Some(ref info) = cache.accessory_info {
                return Ok((
                    info.product_name.clone(),
                    info.firmware_version.clone(),
                    info.serial_number.clone(),
                    info.firmware_build_number,
                    info.display_name.clone(),
                    info.features.clone()
                ));
            }
        }
        
        // Fetch if not cached
        let info = runtime.block_on(async {
            client.get_accessory_info().await
        })?;
        
        // Update cache
        {
            let mut cache = cached_state.write().unwrap();
            cache.accessory_info = Some(info.clone());
        }
        
        Ok((
            info.product_name,
            info.firmware_version,
            info.serial_number,
            info.firmware_build_number,
            info.display_name,
            info.features
        ))
    }
    
    fn get_settings(&self) -> Result<(u8, u8, u32, u32, u32, u32)> {
        let client = self.client()?;
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        // Try to get from cache first
        {
            let cache = cached_state.read().unwrap();
            if let Some(ref settings) = cache.settings {
                return Ok((
                    settings.power_on_behavior,
                    settings.power_on_brightness,
                    LightState::api_to_kelvin(settings.power_on_temperature),
                    settings.switch_on_duration_ms,
                    settings.switch_off_duration_ms,
                    settings.color_change_duration_ms
                ));
            }
        }
        
        // Fetch if not cached
        let settings = runtime.block_on(async {
            client.get_settings().await
        })?;
        
        // Update cache
        {
            let mut cache = cached_state.write().unwrap();
            cache.settings = Some(settings.clone());
        }
        
        Ok((
            settings.power_on_behavior,
            settings.power_on_brightness,
            LightState::api_to_kelvin(settings.power_on_temperature),
            settings.switch_on_duration_ms,
            settings.switch_off_duration_ms,
            settings.color_change_duration_ms
        ))
    }
    
    fn set_settings(&self, power_on_behavior: u8, power_on_brightness: u8, power_on_kelvin: u32,
                   switch_on_ms: u32, switch_off_ms: u32, color_change_ms: u32) -> Result<bool> {
        let client = self.client()?;
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        let settings = Settings {
            power_on_behavior,
            power_on_brightness,
            power_on_temperature: LightState::kelvin_to_api(power_on_kelvin),
            switch_on_duration_ms: switch_on_ms,
            switch_off_duration_ms: switch_off_ms,
            color_change_duration_ms: color_change_ms,
        };
        
        // Update cache immediately
        {
            let mut cache = cached_state.write().unwrap();
            cache.settings = Some(settings.clone());
        }
        
        // Send command asynchronously
        std::thread::spawn(move || {
            runtime.block_on(async {
                let _ = client.set_settings(&settings).await;
            });
        });
        
        Ok(true)
    }
    
    fn set_power_on_settings(&self, behavior: u8, brightness: u8, kelvin: u32) -> Result<bool> {
        // Get current settings
        let (_, _, _, switch_on, switch_off, color_change) = self.get_settings()?;
        
        // Update only power-on related settings
        self.set_settings(behavior, brightness, kelvin, switch_on, switch_off, color_change)
    }
    
    fn identify(&self) -> Result<bool> {
        let client = self.client()?;
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
        
        let client = self.client()?;
        let cached_state = self.cached_state.clone();
        let runtime = self.runtime.clone();
        
        let api_temp = LightState::kelvin_to_api(kelvin);
        
        // Update cache immediately
        {
            let mut cache = cached_state.write().unwrap();
            for i in 0..cache.is_on.len() {
                cache.is_on[i] = true;
                cache.brightness[i] = brightness;
                cache.temperature[i] = api_temp;
            }
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

fn register_interface(cr: &mut Crossroads, _service: Arc<LightService>) -> dbus_crossroads::IfaceToken<Arc<LightService>> {
    cr.register(DBUS_INTERFACE, |b: &mut IfaceBuilder<Arc<LightService>>| {
        // Basic controls
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
        
        // Multi-light controls
        b.method("TurnOnLight", ("light_index",), ("success",), |_, service, (light_index,): (u8,)| {
            service.turn_on_light(Some(light_index))
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("TurnOffLight", ("light_index",), ("success",), |_, service, (light_index,): (u8,)| {
            service.turn_off_light(Some(light_index))
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("ToggleLight", ("light_index",), ("success",), |_, service, (light_index,): (u8,)| {
            service.toggle_light(light_index)
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        // Brightness controls
        b.method("SetBrightness", ("brightness",), ("success",), |_, service, (brightness,): (u8,)| {
            service.set_brightness(brightness)
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("SetBrightnessLight", ("brightness", "light_index"), ("success",), |_, service, (brightness, light_index): (u8, u8)| {
            service.set_brightness_light(brightness, Some(light_index))
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        // Temperature controls
        b.method("SetTemperature", ("kelvin",), ("success",), |_, service, (kelvin,): (u32,)| {
            service.set_temperature(kelvin)
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("SetTemperatureLight", ("kelvin", "light_index"), ("success",), |_, service, (kelvin, light_index): (u32, u8)| {
            service.set_temperature_light(kelvin, Some(light_index))
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        // Status methods
        b.method("GetStatus", (), ("is_on", "brightness", "temperature"), |_, service, _: ()| {
            service.get_status()
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("GetNumLights", (), ("num_lights",), |_, service, _: ()| {
            service.get_num_lights()
                .map(|n| (n,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("GetAllLightsStatus", (), ("is_on_array", "brightness_array", "temperature_array"), |_, service, _: ()| {
            service.get_all_lights_status()
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("GetLightStatus", ("light_index",), ("is_on", "brightness", "temperature"), |_, service, (light_index,): (u8,)| {
            service.get_light_status(light_index)
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        // Accessory info
        b.method("GetAccessoryInfo", (), ("product_name", "firmware_version", "serial_number", "firmware_build", "display_name", "features"), |_, service, _: ()| {
            service.get_accessory_info()
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        // Settings management
        b.method("GetSettings", (), ("power_on_behavior", "power_on_brightness", "power_on_temperature", "switch_on_ms", "switch_off_ms", "color_change_ms"), |_, service, _: ()| {
            service.get_settings()
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("SetSettings", ("power_on_behavior", "power_on_brightness", "power_on_temperature", "switch_on_ms", "switch_off_ms", "color_change_ms"), ("success",), 
            |_, service, (behavior, brightness, temp, on_ms, off_ms, color_ms): (u8, u8, u32, u32, u32, u32)| {
            service.set_settings(behavior, brightness, temp, on_ms, off_ms, color_ms)
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        b.method("SetPowerOnSettings", ("behavior", "brightness", "temperature"), ("success",), 
            |_, service, (behavior, brightness, temp): (u8, u8, u32)| {
            service.set_power_on_settings(behavior, brightness, temp)
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        // Other methods
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

        // Light selection / discovery
        b.method("Discover", ("timeout_secs",), ("lights_json",), |_, service, (timeout,): (u32,)| {
            service.discover(timeout)
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });

        b.method("GetActiveLight", (), ("ip", "port"), |_, service, _: ()| {
            Ok(service.get_active_light())
        });

        b.method("SetActiveLight", ("ip", "port"), ("success",), |_, service, (ip, port): (String, u16)| {
            service.set_active_light(ip, port)
                .map(|s| (s,))
                .map_err(|e| dbus::MethodErr::failed(&e))
        });
        
        // Properties (backward compatibility)
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
        
        b.property("NumLights")
            .get(|_, service| {
                service.get_num_lights()
                    .map_err(|e| dbus::MethodErr::failed(&e))
            });
    })
}

fn main() -> Result<()> {
    env_logger::init();

    let initial = resolve_active(None, None);

    info!("Starting Holikeyz Ring Light D-Bus service");
    match &initial {
        Some(a) => info!(
            "Active light loaded: {}:{} (from {} or RING_LIGHT_IP)",
            a.ip, a.port, active_config_path().display()
        ),
        None => info!("No active light configured. Waiting for Discover + SetActiveLight."),
    }

    let service = Arc::new(LightService::new(initial));
    
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