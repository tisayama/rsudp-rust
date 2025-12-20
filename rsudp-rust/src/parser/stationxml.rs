use std::collections::HashMap;
use std::process::Command;
use tracing::{info, warn};

pub fn fetch_sensitivity(net: &str, sta: &str) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
    let servers = [
        "https://data.raspberryshake.org/fdsnws/station/1/query",
        "https://service.iris.edu/fdsnws/station/1/query",
    ];
    
    for server in servers {
        // Raspberry Shake uses 'station=' instead of standard 'sta=' in some contexts, 
        // though standard FDSN allows 'sta'. We try both or handle it.
        let sta_param = if server.contains("raspberryshake") { "station" } else { "sta" };
        
        let url = format!(
            "{}?net={}&{}={}&level=channel&format=xml",
            server, net, sta_param, sta
        );
        
        info!("Fetching StationXML from: {}", url);
        
        let output = match Command::new("curl")
            .args(["-s", "-L", "-m", "15", &url])
            .output() {
                Ok(o) => o,
                Err(e) => {
                    warn!("curl execution failed for {}: {}", server, e);
                    continue;
                }
            };
            
        if !output.status.success() {
            warn!("Server {} returned status {}", server, output.status);
            continue;
        }
        
        let xml_text = String::from_utf8_lossy(&output.stdout);
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
