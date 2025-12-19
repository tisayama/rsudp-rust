pub mod header;
pub mod mseed;
pub mod steim;

use chrono::{DateTime, Utc};

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

impl TraceSegment {
    pub fn nslc(&self) -> String {
        format!("{}.{}.{}.{}", self.network, self.station, self.location, self.channel)
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
            
            let total_samples: usize = segments.iter()
                .filter(|s| s.channel == "EHZ")
                .map(|s| s.samples.len())
                .sum();
            
            // Expected sample count for this file is around 60,000
            assert!(total_samples > 60000);
        }
    }
}
