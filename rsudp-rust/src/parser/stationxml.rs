use std::collections::HashMap;
use tracing::{info, warn};
use reqwest::Client;

pub async fn fetch_sensitivity(net: &str, sta: &str) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
    let servers = [
        "https://data.raspberryshake.org/fdsnws/station/1/query",
        "https://service.iris.edu/fdsnws/station/1/query",
    ];
    
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    for server in servers {
        // Raspberry Shake uses 'station=' instead of standard 'sta=' in some contexts, 
        // though standard FDSN allows 'sta'. We try both or handle it.
        let sta_param = if server.contains("raspberryshake") { "station" } else { "sta" };
        
        let url = format!(
            "{}?net={}&{}={}&level=channel&format=xml",
            server, net, sta_param, sta
        );
        
        info!("Fetching StationXML from: {}", url);
        
        let response = match client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                warn!("Request failed for {}: {}", server, e);
                continue;
            }
        };
            
        if !response.status().is_success() {
            warn!("Server {} returned status {}", server, response.status());
            continue;
        }
        
        let xml_text = match response.text().await {
            Ok(t) => t,
            Err(e) => {
                warn!("Failed to read response text from {}: {}", server, e);
                continue;
            }
        };

        if xml_text.is_empty() || xml_text.contains("No results") || xml_text.contains("404 Not Found") {
            warn!("Server {} returned no data for {}.{}", server, net, sta);
            continue;
        }

        let mut sensitivities = HashMap::new();
        // Manual parsing to handle XML without complex dependencies
        let channels = xml_text.split("<Channel");
        for ch_block in channels.skip(1) {
            if let Some(code_idx) = ch_block.find("code=\"") {
                let start = code_idx + 6;
                if let Some(end) = ch_block[start..].find("\"") {
                    let code = &ch_block[start..start+end];
                    
                    if let Some(sens_idx) = ch_block.find("<InstrumentSensitivity>") {
                        let sens_block = &ch_block[sens_idx..];
                        if let Some(val_start_idx) = sens_block.find("<Value>") {
                            let val_start = val_start_idx + 7;
                            if let Some(val_end_idx) = sens_block[val_start..].find("</Value>") {
                                let val_str = &sens_block[val_start..val_start + val_end_idx];
                                if let Ok(val) = val_str.trim().parse::<f64>() {
                                    info!("Found sensitivity for {}.{}: {} from {}", sta, code, val, server);
                                    sensitivities.insert(code.to_string(), val);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if !sensitivities.is_empty() {
            return Ok(sensitivities);
        }
    }
    
    Err("Could not find station metadata on any known FDSN server".into())
}
