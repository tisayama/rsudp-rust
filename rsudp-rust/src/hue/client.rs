use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct HueClient {
    client: Client,
    base_url: String,
    app_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LinkResponse {
    success: Option<LinkSuccess>,
    error: Option<LinkError>,
}

#[derive(Debug, Deserialize)]
struct LinkSuccess {
    username: String,
}

#[derive(Debug, Deserialize)]
struct LinkError {
    description: String,
}

#[derive(Serialize)]
struct DeviceType {
    devicetype: String,
    generateclientkey: bool,
}

#[derive(Debug, Deserialize)]
pub struct ResourceData {
    pub id: String,
    pub r#type: String,
    pub metadata: Option<Metadata>,
    pub on: Option<OnState>,
    pub dimming: Option<Dimming>,
    pub color: Option<Color>,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct OnState {
    pub on: bool,
}

#[derive(Debug, Deserialize)]
pub struct Dimming {
    pub brightness: f64,
}

#[derive(Debug, Deserialize)]
pub struct Color {
    pub xy: Xy,
}

#[derive(Debug, Deserialize)]
pub struct Xy {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Deserialize)]
struct GetResourceResponse {
    data: Vec<ResourceData>,
}

impl HueClient {
    pub fn new(ip: &str, app_key: Option<String>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true) // Hue uses self-signed certs
            .timeout(Duration::from_secs(5))
            .build()?;

        let base_url = if ip.contains(':') {
            format!("https://[{}]", ip)
        } else {
            format!("https://{}", ip)
        };

        Ok(Self {
            client,
            base_url,
            app_key,
        })
    }

    pub async fn register_app(&self) -> Result<String, String> {
        let url = format!("{}/api", self.base_url);
        let body = DeviceType {
            devicetype: "rsudp_rust#cli".to_string(),
            generateclientkey: true,
        };

        let resp = self.client.post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let results: Vec<LinkResponse> = resp.json().await.map_err(|e| e.to_string())?;

        if let Some(first) = results.first() {
            if let Some(success) = &first.success {
                return Ok(success.username.clone());
            }
            if let Some(error) = &first.error {
                return Err(error.description.clone());
            }
        }
        Err("Unknown error".to_string())
    }

    pub async fn get_lights(&self) -> Result<Vec<ResourceData>, String> {
        self.get_resource("light").await
    }

    pub async fn get_rooms(&self) -> Result<Vec<ResourceData>, String> {
        self.get_resource("room").await
    }

    pub async fn get_zones(&self) -> Result<Vec<ResourceData>, String> {
        self.get_resource("zone").await
    }

    async fn get_resource(&self, r_type: &str) -> Result<Vec<ResourceData>, String> {
        let key = self.app_key.as_ref().ok_or("No app key")?;
        let url = format!("{}/clip/v2/resource/{}", self.base_url, r_type);

        let resp = self.client.get(&url)
            .header("hue-application-key", key)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("API Error: {}", resp.status()));
        }

        let body: GetResourceResponse = resp.json().await.map_err(|e| e.to_string())?;
        Ok(body.data)
    }

    pub async fn set_light_state(&self, id: &str, payload: &serde_json::Value) -> Result<(), String> {
        let key = self.app_key.as_ref().ok_or("No app key")?;
        let url = format!("{}/clip/v2/resource/light/{}", self.base_url, id);

        let resp = self.client.put(&url)
            .header("hue-application-key", key)
            .json(payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("API Error: {}", resp.status()));
        }
        Ok(())
    }
}
