use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use crate::hue::config::HueConfig;
use crate::hue::client::HueClient;
use crate::hue::discovery::Discovery;
use tracing::{info, warn, error};
use serde_json::json;
use std::collections::HashMap;

pub mod client;
pub mod config;
pub mod discovery;

pub fn rgb_to_xy(r: u8, g: u8, b: u8) -> (f32, f32) {
    let r_norm = (r as f32 / 255.0).powf(2.4); // Gamma correction
    let g_norm = (g as f32 / 255.0).powf(2.4);
    let b_norm = (b as f32 / 255.0).powf(2.4);

    // RGB to XYZ (Wide RGB D65)
    let x = r_norm * 0.664511 + g_norm * 0.154324 + b_norm * 0.162028;
    let y = r_norm * 0.283881 + g_norm * 0.729798 + b_norm * 0.086320;
    let z = r_norm * 0.000088 + g_norm * 0.065924 + b_norm * 0.918157;

    let sum = x + y + z;
    if sum == 0.0 {
        return (0.0, 0.0);
    }

    (x / sum, y / sum)
}

#[derive(Clone)]
struct StoredState {
    on: bool,
    xy: Option<(f32, f32)>,
    brightness: Option<f64>,
}

#[derive(Clone)]
pub struct HueIntegration {
    config: HueConfig,
    client: Arc<Mutex<Option<HueClient>>>,
    // Map of light ID to its state BEFORE the alert started
    pre_alert_states: Arc<Mutex<HashMap<String, StoredState>>>,
    // Token to manage preemption
    reset_token: Arc<Mutex<u64>>,
}

impl HueIntegration {
    pub fn new(config: HueConfig) -> Self {
        Self {
            config,
            client: Arc::new(Mutex::new(None)),
            pre_alert_states: Arc::new(Mutex::new(HashMap::new())),
            reset_token: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn start(&self) {
        if !self.config.enabled {
            return;
        }

        let integration = self.clone();
        tokio::spawn(async move {
            integration.discovery_loop().await;
        });
    }

    async fn discovery_loop(&self) {
        loop {
            // Run synchronous mDNS discovery on a blocking thread so it doesn't
            // starve the tokio executor (recv_timeout is blocking I/O).
            let bridge_id = self.config.bridge_id.clone();
            let result = tokio::task::spawn_blocking(move || {
                Discovery::find_bridge_blocking(Duration::from_secs(5), bridge_id)
            }).await.ok().flatten();

            if let Some((_id, ip)) = result {
                match HueClient::new(&ip.to_string(), Some(self.config.app_key.clone())) {
                    Ok(c) => {
                        let mut guard = self.client.lock().await;
                        *guard = Some(c);
                        info!("Hue Integration connected to Bridge at {}", ip);
                        // Bridge found and connected — no need for aggressive re-discovery.
                        // Re-check every 10 minutes in case bridge IP changes.
                        tokio::time::sleep(Duration::from_secs(600)).await;
                        continue;
                    }
                    Err(e) => error!("Failed to create Hue client: {}", e),
                }
            } else {
                warn!("Hue Bridge not found. Retrying in 60s...");
            }
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    async fn capture_current_state(&self) {
        let guard = self.client.lock().await;
        if let Some(client) = &*guard {
            match client.get_lights().await {
                Ok(lights) => {
                    let mut states = self.pre_alert_states.lock().await;
                    for light in lights {
                        // Only capture state for target lights
                        if self.config.target_ids.contains(&light.id) {
                                                        // Extract actual state from ResourceData
                                                        let on = light.on.map(|o| o.on).unwrap_or(false);
                                                        let xy = light.color.map(|c| (c.xy.x, c.xy.y));
                                                        let brightness = light.dimming.map(|d| d.brightness);
                                                        
                                                        info!("Captured state for {}: On={}, XY={:?}, Bri={:?}", light.id, on, xy, brightness);
                            
                                                        states.insert(light.id.clone(), StoredState {
                                                            on,
                                                            xy,
                                                            brightness,
                                                        });
                                                    }
                                                }
                                            }
                                            Err(e) => error!("Failed to capture state: {}", e),
                                        }
                                    }
                                }
                            
                                pub async fn trigger_alert(&self) {
                                    if !self.config.enabled { return; }
                                    
                                    // Increment token to invalidate any running resets
                                    let mut token = self.reset_token.lock().await;
                                    *token += 1;
                            
                                    // Capture state (first time only)
                                    // If an alert is already active, we shouldn't overwrite the *original* pre-alert state. 
                                    {
                                        let states = self.pre_alert_states.lock().await;
                                        if states.is_empty() {
                                            drop(states); // release lock
                                            self.capture_current_state().await;
                                        }
                                    }
                                    
                                    let guard = self.client.lock().await;
                                    if let Some(client) = &*guard {
                                        info!("Sending Yellow Pulse to Hue lights...");
                                        let (x, y) = rgb_to_xy(255, 255, 0); // Yellow
                                        let payload = json!({
                                            "on": { "on": true },
                                            "alert": { "action": "breathe" },
                                            "color": { "xy": { "x": x, "y": y } }
                                        });
                            
                                        for id in &self.config.target_ids {
                                            if let Err(e) = client.set_light_state(id, &payload).await {
                                                error!("Failed to alert light {}: {}", id, e);
                                            }
                                        }
                                    }
                                }
                            
                                pub async fn reset_alert(&self, max_intensity: f64) {
                                    if !self.config.enabled { return; }
                            
                                    let color = self.get_jma_color(max_intensity);
                                    let (x, y) = rgb_to_xy(color.0, color.1, color.2);
                                    
                                    // Get current token
                                    let current_token = *self.reset_token.lock().await;
                            
                                    let guard = self.client.lock().await;
                                    if let Some(client) = &*guard {
                                        info!("Sending Reset Pulse (Intensity {:.1}) to Hue lights...", max_intensity);
                                        
                                        let payload = json!({
                                            "on": { "on": true },
                                            "alert": { "action": "breathe" },
                                            "color": { "xy": { "x": x, "y": y } }
                                        });
                            
                                        for id in &self.config.target_ids {
                                            if let Err(e) = client.set_light_state(id, &payload).await {
                                                error!("Failed to pulse light {}: {}", id, e);
                                            }
                                        }
                            
                                        // Spawn 20s loop/wait
                                        let target_ids = self.config.target_ids.clone();
                                        let client_clone = self.client.clone();
                                        let token_clone = self.reset_token.clone();
                                        let states_clone = self.pre_alert_states.clone();
                                        
                                        tokio::spawn(async move {
                                            // Loop pulse for 20s (since breathe is short)
                                            for _ in 0..20 {
                                                tokio::time::sleep(Duration::from_secs(1)).await;
                                                // Check preemption
                                                if *token_clone.lock().await != current_token {
                                                    return; // Preempted by new trigger
                                                }
                                            }
                            
                                                                                            // Restore state
                            
                                                                                            let guard = client_clone.lock().await;
                            
                                                                                            if let Some(client) = &*guard {
                            
                                                                                                let mut states = states_clone.lock().await;
                            
                                                                                                for id in &target_ids {
                            
                                                                                                    // Step 1: Stop alert explicitly (with on:true to satisfy potential API constraints)
                            
                                                                                                    let stop_payload = json!({ 
                            
                                                                                                        "alert": { "action": "stop" }
                            
                                                                                                        // Removed "on" here to verify if stop works alone now that we know combined didn't revert color properly
                            
                                                                                                        // Actually, let's try sending just stop again, but handle the error gracefully?
                            
                                                                                                        // No, user said 400 error.
                            
                                                                                                        // Let's try sending on:true with stop.
                            
                                                                                                    });
                            
                                                                                                    // Actually, let's look at Step 2 below. We will send stop+color there?
                            
                                                                                                    // No, we want to split them.
                            
                                                                                                    
                            
                                                                                                    // Retry splitting, but maybe the 400 was because the light was off?
                            
                                                                                                    // Or maybe just try sending a benign "on" command first?
                            
                                                                                                    
                            
                                                                                                    // Revised Strategy:
                            
                                                                                                    // 1. Send { alert: { action: stop } } -> Ignore 400 error.
                            
                                                                                                    // 2. Wait.
                            
                                                                                                    // 3. Send Color/On.
                            
                                                                                                    
                            
                                                                                                    let _ = client.set_light_state(id, &stop_payload).await;
                            
                                                                                                }
                            
                                                                                                
                            
                                                                                                // Small delay
                            
                                                                                                tokio::time::sleep(Duration::from_millis(500)).await;
                            
                                                                            
                            
                                                                                                for id in &target_ids {
                            
                                                                                                    // Step 2: Restore color/on/dimming
                            
                                                                                                    if let Some(state) = states.get(id) {
                            
                                                                                                        info!("Restoring state for {}: On={}, XY={:?}", id, state.on, state.xy);
                            
                                                                                                        if state.on {
                            
                                                                                                            let mut json_map = serde_json::Map::new();
                            
                                                                                                            json_map.insert("on".to_string(), json!({ "on": true }));
                            
                                                                                                            
                            
                                                                                                            if let Some((x, y)) = state.xy {
                            
                                                                                                                json_map.insert("color".to_string(), json!({ "xy": { "x": x, "y": y } }));
                            
                                                                                                            }
                            
                                                                                                            if let Some(bri) = state.brightness {
                            
                                                                                                                json_map.insert("dimming".to_string(), json!({ "brightness": bri }));
                            
                                                                                                            }
                            
                                                                                                            let restore_payload = serde_json::Value::Object(json_map);
                            
                                                                                                            let _ = client.set_light_state(id, &restore_payload).await;
                            
                                                                                                        } else {
                            
                                                                                                            // If it was off, just turn it off
                            
                                                                                                            let off_payload = json!({ "on": { "on": false } });
                            
                                                                                                            let _ = client.set_light_state(id, &off_payload).await;
                            
                                                                                                        }
                            
                                                                                                    } else {
                            
                                                                                                        warn!("No state found for {}, assuming no further action needed", id);
                            
                                                                                                    }
                            
                                                                                                }
                            
                                                                                                states.clear(); // Clear state after restore
                            
                                                                                            }            });
        }
    }

    fn get_jma_color(&self, intensity: f64) -> (u8, u8, u8) {
        // Int 7: 180, 0, 104
        if intensity >= 6.5 { return (180, 0, 104); }
        // Int 6+: 165, 0, 33
        if intensity >= 6.0 { return (165, 0, 33); }
        // Int 6-: 255, 40, 0
        if intensity >= 5.5 { return (255, 40, 0); }
        // Int 5+: 255, 153, 0
        if intensity >= 5.0 { return (255, 153, 0); }
        // Int 5-: 255, 230, 0
        if intensity >= 4.5 { return (255, 230, 0); }
        // Int 4: 250, 230, 150
        if intensity >= 3.5 { return (250, 230, 150); }
        // Int 3: 0, 65, 255
        if intensity >= 2.5 { return (0, 65, 255); }
        // Int 2: 0, 170, 255
        if intensity >= 1.5 { return (0, 170, 255); }
        // Int 1: 242, 242, 255
        (242, 242, 255)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_xy() {
        // Red
        let (x, y) = rgb_to_xy(255, 0, 0);
        assert!(x > 0.6 && y > 0.2); // Rough check for Red area

        // Green
        let (x, y) = rgb_to_xy(0, 255, 0);
        assert!(x < 0.3 && y > 0.6); // Rough check for Green area

        // Blue
        let (x, y) = rgb_to_xy(0, 0, 255);
        assert!(x < 0.2 && y < 0.1); // Rough check for Blue area
        
        // Black (Off)
        let (x, y) = rgb_to_xy(0, 0, 0);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_jma_color_mapping() {
        let integration = HueIntegration::new(HueConfig::default());
        
        // Intensity 7
        assert_eq!(integration.get_jma_color(6.8), (180, 0, 104));
        // Intensity 4
        assert_eq!(integration.get_jma_color(3.7), (250, 230, 150));
        // Intensity 1
        assert_eq!(integration.get_jma_color(1.0), (242, 242, 255));
    }
}