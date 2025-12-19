use chrono::{DateTime, Utc};

pub mod header;
pub mod steim;
pub mod mseed;

#[derive(Debug, Clone)]
pub struct TraceSegment {
    pub network: String,
    pub station: String,
    pub location: String,
    pub channel: String,
    pub starttime: DateTime<Utc>,
    pub sampling_rate: f64,
    pub samples: Vec<f64>,
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
    fn test_regression_pure_rust_parser() {
        // Test Steim2 file
        let path2 = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../references/mseed/fdsnws.mseed");
        
        println!("Regression test: Checking file {:?}", path2);
        if path2.exists() {
            let data = std::fs::read(&path2).unwrap();
            let segments = mseed::parse_mseed_stream(&data).expect("Should parse MiniSEED stream");
            assert!(!segments.is_empty());
            
            let expected_first_10 = [16895.0, 16761.0, 16761.0, 16795.0, 16864.0, 16708.0, 16832.0, 16797.0, 16740.0, 17002.0];
            let first_seg = segments.iter().find(|s| s.nslc() == "AM.R6E01.00.EHZ").unwrap();
            for i in 0..10 {
                assert_eq!(first_seg.samples[i], expected_first_10[i], "Sample {} mismatch in Steim2", i);
            }
            let total_ehz_samples: usize = segments.iter()
                .filter(|s| s.nslc() == "AM.R6E01.00.EHZ")
                .map(|s| s.samples.len())
                .sum();
            assert_eq!(total_ehz_samples, 60246);
            println!("Steim2 verification passed!");
        }

        // Test Steim1 file
        let path1 = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../references/obspy/obspy/io/mseed/tests/data/encoding/int32_Steim1_bigEndian.mseed");
        
        println!("Regression test: Checking file {:?}", path1);
        if path1.exists() {
            let data = std::fs::read(&path1).unwrap();
            let segments = mseed::parse_mseed_stream(&data).expect("Should parse MiniSEED stream");
            assert!(!segments.is_empty());
            
            let expected_first_10 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
            let first_seg = &segments[0];
            assert_eq!(first_seg.nslc(), "XX.TEST..BHE");
            for i in 0..10 {
                assert_eq!(first_seg.samples[i], expected_first_10[i], "Sample {} mismatch in Steim1", i);
            }
            assert_eq!(first_seg.samples.len(), 50);
            println!("Steim1 verification passed!");
        }
    }
}