use crate::{error::Result, models::*};
use reqwest::{Client, StatusCode};
use std::time::Duration;
use log::{debug, info};

pub struct RingLightClient {
    client: Client,
    base_url: String,
}

impl RingLightClient {
    pub fn new(ip: &str, port: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(500))  // Very aggressive timeout
            .connect_timeout(Duration::from_millis(200))  // Fast connect
            .pool_idle_timeout(Duration::from_secs(300))  // Keep connections alive longer
            .pool_max_idle_per_host(5)  // More connection reuse
            .tcp_nodelay(true)  // Disable Nagle's algorithm for lower latency
            .tcp_keepalive(Some(Duration::from_secs(30)))  // Keep TCP alive
            .build()
            .unwrap_or_else(|_| Client::new());
            
        Self {
            client,
            base_url: format!("http://{}:{}/elgato", ip, port), // API endpoint remains the same
        }
    }
    
    pub async fn get_lights(&self) -> Result<LightResponse> {
        let url = format!("{}/lights", self.base_url);
        debug!("GET {}", url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
            
        if response.status() != StatusCode::OK {
            return Err(crate::error::HolikeyzError::DeviceNotFound(self.base_url.clone()));
        }
        
        let light_response = response.json::<LightResponse>().await?;
        debug!("Received: {:?}", light_response);
        Ok(light_response)
    }
    
    pub async fn set_lights(&self, state: &LightState) -> Result<LightResponse> {
        let url = format!("{}/lights", self.base_url);
        debug!("PUT {} with {:?}", url, state);
        
        let request_body = LightResponse {
            number_of_lights: 1,
            lights: vec![state.clone()],
        };
        
        let response = self.client
            .put(&url)
            .json(&request_body)
            .send()
            .await?;
            
        if response.status() != StatusCode::OK {
            return Err(crate::error::HolikeyzError::DeviceNotFound(self.base_url.clone()));
        }
        
        let light_response = response.json::<LightResponse>().await?;
        info!("Light state updated: {:?}", light_response);
        Ok(light_response)
    }
    
    pub async fn turn_on(&self) -> Result<LightResponse> {
        let mut current = self.get_lights().await?;
        if let Some(light) = current.lights.first_mut() {
            light.set_on(true);
            self.set_lights(light).await
        } else {
            let mut state = LightState::new();
            state.set_on(true);
            self.set_lights(&state).await
        }
    }
    
    pub async fn turn_off(&self) -> Result<LightResponse> {
        let mut current = self.get_lights().await?;
        if let Some(light) = current.lights.first_mut() {
            light.set_on(false);
            self.set_lights(light).await
        } else {
            let mut state = LightState::new();
            state.set_on(false);
            self.set_lights(&state).await
        }
    }
    
    pub async fn set_brightness(&self, brightness: u8) -> Result<LightResponse> {
        if brightness > 100 {
            return Err(crate::error::HolikeyzError::InvalidBrightness(brightness));
        }
        
        // Quick optimization: use cached state if available
        let mut state = LightState::new();
        state.set_on(true);
        state.brightness = brightness;
        state.temperature = 213;  // Default middle temperature
        self.set_lights(&state).await
    }
    
    pub async fn set_temperature_kelvin(&self, kelvin: u32) -> Result<LightResponse> {
        let api_value = LightState::kelvin_to_api(kelvin);
        self.set_temperature(api_value).await
    }
    
    pub async fn set_temperature(&self, temperature: u16) -> Result<LightResponse> {
        if !(143..=344).contains(&temperature) {
            return Err(crate::error::HolikeyzError::InvalidTemperature(temperature));
        }
        
        let mut current = self.get_lights().await?;
        if let Some(light) = current.lights.first_mut() {
            light.temperature = temperature;
            self.set_lights(light).await
        } else {
            let mut state = LightState::new();
            state.temperature = temperature;
            self.set_lights(&state).await
        }
    }
    
    pub async fn get_accessory_info(&self) -> Result<AccessoryInfo> {
        let url = format!("{}/accessory-info", self.base_url);
        debug!("GET {}", url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
            
        if response.status() != StatusCode::OK {
            return Err(crate::error::HolikeyzError::DeviceNotFound(self.base_url.clone()));
        }
        
        let info = response.json::<AccessoryInfo>().await?;
        debug!("Accessory info: {:?}", info);
        Ok(info)
    }
    
    pub async fn get_settings(&self) -> Result<Settings> {
        let url = format!("{}/settings", self.base_url);
        debug!("GET {}", url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
            
        if response.status() != StatusCode::OK {
            return Err(crate::error::HolikeyzError::DeviceNotFound(self.base_url.clone()));
        }
        
        let settings = response.json::<Settings>().await?;
        debug!("Settings: {:?}", settings);
        Ok(settings)
    }
    
    pub async fn set_settings(&self, settings: &Settings) -> Result<Settings> {
        let url = format!("{}/settings", self.base_url);
        debug!("PUT {} with {:?}", url, settings);
        
        let response = self.client
            .put(&url)
            .json(settings)
            .send()
            .await?;
            
        if response.status() != StatusCode::OK {
            return Err(crate::error::HolikeyzError::DeviceNotFound(self.base_url.clone()));
        }
        
        let updated_settings = response.json::<Settings>().await?;
        info!("Settings updated: {:?}", updated_settings);
        Ok(updated_settings)
    }
    
    pub async fn identify(&self) -> Result<()> {
        let url = format!("{}/identify", self.base_url);
        debug!("POST {}", url);
        
        let response = self.client
            .post(&url)
            .send()
            .await?;
            
        if response.status() != StatusCode::OK {
            return Err(crate::error::HolikeyzError::DeviceNotFound(self.base_url.clone()));
        }
        
        info!("Light identified (should be flashing)");
        Ok(())
    }
}