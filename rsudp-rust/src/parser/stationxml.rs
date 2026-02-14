use std::collections::HashMap;
use tracing::{info, warn};
use reqwest::Client;

/// Instrument response for a single channel (poles/zeros representation).
/// Used for frequency-domain deconvolution matching obspy's remove_response().
#[derive(Debug, Clone)]
pub struct ChannelResponse {
    /// Complex zeros (real, imaginary) in rad/s
    pub zeros: Vec<(f64, f64)>,
    /// Complex poles (real, imaginary) in rad/s
    pub poles: Vec<(f64, f64)>,
    /// Normalization factor (A0) — ensures response = 1.0 at normalization frequency
    pub normalization_factor: f64,
    /// Stage gain (total gain including sensor + electronics)
    pub stage_gain: f64,
    /// Overall instrument sensitivity (counts per physical unit) — for fallback
    pub sensitivity: f64,
}

/// Fetch full instrument response (poles/zeros) from FDSN for frequency-domain deconvolution.
/// Falls back to sensitivity-only if response parsing fails.
pub async fn fetch_response(net: &str, sta: &str) -> Result<HashMap<String, ChannelResponse>, Box<dyn std::error::Error>> {
    let servers = [
        "https://data.raspberryshake.org/fdsnws/station/1/query",
        "https://service.iris.edu/fdsnws/station/1/query",
    ];

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    for server in servers {
        let sta_param = if server.contains("raspberryshake") { "station" } else { "sta" };

        let url = format!(
            "{}?net={}&{}={}&level=resp&format=xml",
            server, net, sta_param, sta
        );

        info!("Fetching StationXML (level=resp) from: {}", url);

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

        let responses = parse_response_xml(&xml_text, sta);
        if !responses.is_empty() {
            return Ok(responses);
        }
    }

    Err("Could not find instrument response on any known FDSN server".into())
}

/// Parse StationXML (level=resp) to extract poles/zeros for each channel.
fn parse_response_xml(xml: &str, sta: &str) -> HashMap<String, ChannelResponse> {
    let mut responses = HashMap::new();

    let channels = xml.split("<Channel");
    for ch_block in channels.skip(1) {
        // Extract channel code
        let code = match extract_attr(ch_block, "code=\"") {
            Some(c) => c,
            None => continue,
        };

        // Extract InstrumentSensitivity
        let sensitivity = extract_xml_value(ch_block, "<InstrumentSensitivity>", "<Value>")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        if sensitivity <= 0.0 {
            continue;
        }

        // Find the PolesZeros stage (Stage 1 typically)
        if let Some(pz_idx) = ch_block.find("<PolesZeros") {
            let pz_block = &ch_block[pz_idx..];
            let pz_end = pz_block.find("</PolesZeros>").unwrap_or(pz_block.len());
            let pz_block = &pz_block[..pz_end];

            // Parse normalization factor
            let a0 = extract_xml_value(pz_block, "<NormalizationFactor>", "")
                .or_else(|| extract_simple_tag(pz_block, "NormalizationFactor"))
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(1.0);

            // Parse stage gain (from the stage's StageGain/Value)
            let stage_gain = if let Some(sg_idx) = pz_block.find("<StageGain>") {
                let sg_block = &pz_block[sg_idx..];
                extract_xml_value(sg_block, "<Value>", "")
                    .or_else(|| extract_simple_tag(sg_block, "Value"))
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(sensitivity)
            } else {
                sensitivity
            };

            // Parse zeros
            let zeros = parse_complex_values(pz_block, "Zero");
            // Parse poles
            let poles = parse_complex_values(pz_block, "Pole");

            info!(
                "Found response for {}.{}: {} zeros, {} poles, A0={}, gain={}, sensitivity={}",
                sta, code, zeros.len(), poles.len(), a0, stage_gain, sensitivity
            );

            responses.insert(code.to_string(), ChannelResponse {
                zeros,
                poles,
                normalization_factor: a0,
                stage_gain,
                sensitivity,
            });
        } else {
            // No PolesZeros stage — store sensitivity-only response
            info!("No PolesZeros stage for {}.{}, using sensitivity-only: {}", sta, code, sensitivity);
            responses.insert(code.to_string(), ChannelResponse {
                zeros: vec![],
                poles: vec![],
                normalization_factor: 1.0,
                stage_gain: sensitivity,
                sensitivity,
            });
        }
    }

    responses
}

/// Parse Zero or Pole elements from a PolesZeros block.
/// Format: <Zero number="0"><Real>0</Real><Imaginary>0</Imaginary></Zero>
fn parse_complex_values(block: &str, tag: &str) -> Vec<(f64, f64)> {
    let mut values = Vec::new();
    let open_tag = format!("<{}", tag);
    let close_tag = format!("</{}>", tag);

    let mut search_from = 0;
    while let Some(start) = block[search_from..].find(&open_tag) {
        let abs_start = search_from + start;
        if let Some(end) = block[abs_start..].find(&close_tag) {
            let element = &block[abs_start..abs_start + end + close_tag.len()];

            let real = extract_simple_tag(element, "Real")
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            let imag = extract_simple_tag(element, "Imaginary")
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            values.push((real, imag));
            search_from = abs_start + end + close_tag.len();
        } else {
            break;
        }
    }

    values
}

/// Extract a simple XML tag value: <Tag>value</Tag>
fn extract_simple_tag(block: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    if let Some(start_idx) = block.find(&open) {
        let val_start = start_idx + open.len();
        if let Some(end_idx) = block[val_start..].find(&close) {
            return Some(block[val_start..val_start + end_idx].trim().to_string());
        }
    }
    None
}

/// Extract value from nested XML: find outer_tag, then inner_tag within it
fn extract_xml_value(block: &str, outer_tag: &str, inner_tag: &str) -> Option<String> {
    if outer_tag.is_empty() {
        return extract_simple_tag(block, inner_tag.trim_start_matches('<').trim_end_matches('>'));
    }
    if let Some(outer_idx) = block.find(outer_tag) {
        let sub = &block[outer_idx + outer_tag.len()..];
        if inner_tag.is_empty() {
            // No inner tag — value directly after outer tag
            if let Some(end) = sub.find('<') {
                return Some(sub[..end].trim().to_string());
            }
        } else if let Some(inner_idx) = sub.find(inner_tag) {
            let val_start = inner_idx + inner_tag.len();
            if let Some(end) = sub[val_start..].find("</") {
                return Some(sub[val_start..val_start + end].trim().to_string());
            }
        }
    }
    None
}

/// Extract an XML attribute value: code="EHZ" → "EHZ"
fn extract_attr(block: &str, attr_prefix: &str) -> Option<String> {
    if let Some(idx) = block.find(attr_prefix) {
        let start = idx + attr_prefix.len();
        if let Some(end) = block[start..].find('"') {
            return Some(block[start..start + end].to_string());
        }
    }
    None
}

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
