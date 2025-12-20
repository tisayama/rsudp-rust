pub mod header;
pub mod mseed;
pub mod steim;
pub mod stationxml;

use chrono::{DateTime, TimeZone, Utc};

#[derive(Debug, Clone)]
pub struct TraceSegment {
    pub network: String,
    pub station: String,
    pub location: String,
    pub channel: String,
    pub starttime: DateTime<Utc>,
    pub samples: Vec<f64>,
    pub sampling_rate: f64,
}

pub fn parse_any(data: &[u8]) -> Result<Vec<TraceSegment>, Box<dyn std::error::Error>> {
    // Try JSON first if it starts with '['
    if data.starts_with(b"[")
        && let Ok(values) = serde_json::from_slice::<Vec<serde_json::Value>>(data)
        && values.len() >= 3
    {
        let channel = values[0].as_str().unwrap_or("UNK").to_string();
        let timestamp = values[1].as_f64().unwrap_or(0.0);
        let starttime = Utc
            .timestamp_opt(
                timestamp as i64,
                ((timestamp % 1.0) * 1_000_000_000.0) as u32,
            )
            .unwrap();

        let mut samples = Vec::new();
        for val in values.iter().skip(2) {
            if let Some(s) = val.as_f64() {
                samples.push(s);
            }
        }

        return Ok(vec![TraceSegment {
            network: "XX".to_string(),
            station: "SIM".to_string(),
            location: "00".to_string(),
            channel,
            starttime,
            samples,
            sampling_rate: 100.0, // Assuming 100Hz for rsudp
        }]);
    }

    // Fallback to MiniSEED
    mseed::parse_mseed_record(data)
}

impl TraceSegment {
    pub fn nslc(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.network, self.station, self.location, self.channel
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_parse_fdsnws_mseed() {
        let path = "../references/mseed/fdsnws.mseed";
        if Path::new(path).exists() {
            let segments = mseed::parse_mseed_file(path).expect("Should parse MiniSEED file");
            assert!(!segments.is_empty());

            let first_seg: &TraceSegment = segments.iter().find(|s| s.channel == "EHZ").unwrap();
            assert_eq!(first_seg.station, "R6E01");

            let total_samples: usize = segments
                .iter()
                .filter(|s| s.channel == "EHZ")
                .map(|s| s.samples.len())
                .sum();

            // Expected sample count for this file is around 60,000
            assert!(total_samples > 60000);
        }
    }
}
