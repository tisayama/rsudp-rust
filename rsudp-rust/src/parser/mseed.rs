use crate::parser::TraceSegment;
use crate::parser::header::SeedHeader;
use crate::parser::steim::SteimDecoder;
use tracing::warn;

/// Parses a MiniSEED file containing multiple records (Simulation Mode).
pub fn parse_mseed_file(path: &str) -> Result<Vec<TraceSegment>, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    parse_mseed_stream(&data)
}

/// Parses a stream of MiniSEED records.
pub fn parse_mseed_stream(data: &[u8]) -> Result<Vec<TraceSegment>, String> {
    let mut segments = Vec::new();
    let mut offset = 0;

    while offset + 48 <= data.len() {
        match SeedHeader::parse(&data[offset..]) {
            Ok(header) => {
                let record_size = header.record_size;
                if offset + record_size > data.len() {
                    break;
                }

                let data_offset = header.data_offset as usize;
                if data_offset > 0 && data_offset < record_size {
                    let compressed_data = &data[offset + data_offset..offset + record_size];
                    
                    let result = if header.encoding == 10 {
                        SteimDecoder::decode_steim1(compressed_data, header.num_samples as usize)
                    } else if header.encoding == 11 {
                        SteimDecoder::decode_steim2(compressed_data, header.num_samples as usize)
                    } else {
                        Err(crate::parser::steim::SteimError::InvalidSteimCode(header.encoding))
                    };

                    match result {
                        Ok(samples) => {
                            segments.push(TraceSegment {
                                network: header.network.clone(),
                                station: header.station.clone(),
                                location: header.location.clone(),
                                channel: header.channel.clone(),
                                starttime: header.start_time,
                                sampling_rate: header.sample_rate(),
                                samples: samples.into_iter().map(|x| x as f64).collect(),
                            });
                        }
                        Err(e) => {
                            warn!("Steim decode error at offset {}: {}", offset, e);
                        }
                    }
                }
                offset += record_size;
            }
            Err(_e) => {
                // If header parsing fails at a 512-byte boundary, it might be a malformed record
                // Otherwise we just advance to find the next valid record
                offset += 1;
            }
        }
    }

    Ok(segments)
}

/// Parses a single MiniSEED record from a byte slice (e.g., from UDP).
pub fn parse_mseed_record(data: &[u8]) -> Result<Vec<TraceSegment>, String> {
    parse_mseed_stream(data)
}