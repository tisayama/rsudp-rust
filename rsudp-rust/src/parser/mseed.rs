use crate::parser::TraceSegment;
use mseed::{MSReader, MSRecord, MSControlFlags, MSError};
use chrono::{Utc, TimeZone};

/// Parses a single MiniSEED record from a byte slice (e.g., from UDP).
pub fn parse_mseed_record(data: &[u8]) -> Result<Vec<TraceSegment>, String> {
    let mut buffer = data.to_vec();
    let mut record = MSRecord::parse(&mut buffer, MSControlFlags::MSF_UNPACKDATA)
        .map_err(|e: MSError| e.to_string())?;
    
    // Explicitly decompress data samples
    record.unpack_data().map_err(|e: MSError| e.to_string())?;
    
    // Try to get samples. Steim2 results in i32 after unpacking.
    let samples: Vec<f64> = if let Some(s) = record.data_samples::<i32>() {
        s.iter().map(|&x| x as f64).collect()
    } else if let Some(s) = record.data_samples::<f32>() {
        s.iter().map(|&x| x as f64).collect()
    } else if let Some(s) = record.data_samples::<f64>() {
        s.to_vec()
    } else {
        Vec::new()
    };

    if samples.is_empty() {
        return Ok(Vec::new());
    }

    let network = record.network().map_err(|e: MSError| e.to_string())?.trim().to_string();
    let station = record.station().map_err(|e: MSError| e.to_string())?.trim().to_string();
    let location = record.location().map_err(|e: MSError| e.to_string())?.trim().to_string();
    let channel = record.channel().map_err(|e: MSError| e.to_string())?.trim().to_string();
    
    let odt = record.start_time().map_err(|e: MSError| e.to_string())?;
    let starttime = Utc.timestamp_nanos(odt.unix_timestamp_nanos() as i64);
    let sampling_rate = record.sample_rate_hz();

    Ok(vec![TraceSegment {
        network,
        station,
        location,
        channel,
        starttime,
        sampling_rate,
        samples,
    }])
}

/// Parses a MiniSEED file containing multiple records (Simulation Mode).
pub fn parse_mseed_file(path: &str) -> Result<Vec<TraceSegment>, String> {
    let reader = MSReader::new(path).map_err(|e: MSError| e.to_string())?;
    let mut segments = Vec::new();

    // Use for loop instead of while let Some
    for record_res in reader {
        let mut record = record_res.map_err(|e: MSError| e.to_string())?;
        
        // Decompress Steim/etc.
        record.unpack_data().map_err(|e: MSError| e.to_string())?;
        
        let samples: Vec<f64> = if let Some(s) = record.data_samples::<i32>() {
            s.iter().map(|&x| x as f64).collect()
        } else if let Some(s) = record.data_samples::<f32>() {
            s.iter().map(|&x| x as f64).collect()
        } else if let Some(s) = record.data_samples::<f64>() {
            s.to_vec()
        } else {
            continue;
        };

        if samples.is_empty() { continue; }

        let network = record.network().map_err(|e: MSError| e.to_string())?.trim().to_string();
        let station = record.station().map_err(|e: MSError| e.to_string())?.trim().to_string();
        let location = record.location().map_err(|e: MSError| e.to_string())?.trim().to_string();
        let channel = record.channel().map_err(|e: MSError| e.to_string())?.trim().to_string();
        
        let odt = record.start_time().map_err(|e: MSError| e.to_string())?;
        let starttime = Utc.timestamp_nanos(odt.unix_timestamp_nanos() as i64);
        let sampling_rate = record.sample_rate_hz();

        segments.push(TraceSegment {
            network,
            station,
            location,
            channel,
            starttime,
            sampling_rate,
            samples,
        });
    }

    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_parse_sample_mseed() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../references/mseed/fdsnws.mseed");
        
        let path_str = path.to_str().unwrap();
        if path.exists() {
            let segments = parse_mseed_file(path_str).expect("Failed to parse mseed file");
            assert!(!segments.is_empty(), "No segments were parsed from the file");
            println!("Parsed {} segments from sample file", segments.len());
            for (i, s) in segments.iter().enumerate().take(5) {
                println!("Record {}: NSLC: {}, samples: {}, start: {}", i, s.nslc(), s.samples.len(), s.starttime);
            }
        } else {
            println!("Warning: Sample file not found at {}. Skipping test.", path_str);
        }
    }
}
