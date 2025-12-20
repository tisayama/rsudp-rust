use crate::parser::TraceSegment;
use crate::parser::header::parse_header;
use crate::parser::steim::SteimDecoder;

pub fn parse_single_record(record: &[u8]) -> Result<TraceSegment, Box<dyn std::error::Error>> {
    let header = parse_header(record)?;
    let data_start = header.data_offset as usize;

    if data_start >= 512 {
        return Err("Invalid data offset".into());
    }

    let compressed_data = &record[data_start..512];

    let samples = match header.encoding {
        10 => SteimDecoder::decode_steim1(compressed_data, header.num_samples as usize)?,
        11 => SteimDecoder::decode_steim2(compressed_data, header.num_samples as usize)?,
        _ => {
            return Err(crate::parser::steim::SteimError::InvalidSteimCode(header.encoding).into());
        }
    };

    let samples_f64: Vec<f64> = samples.iter().map(|&x| x as f64).collect();

    let sampling_rate = if header.sample_rate_factor > 0 {
        if header.sample_rate_multiplier > 0 {
            header.sample_rate_factor as f64 * header.sample_rate_multiplier as f64
        } else {
            header.sample_rate_factor as f64 / (-header.sample_rate_multiplier as f64)
        }
    } else {
        100.0
    };

    Ok(TraceSegment {
        network: header.network,
        station: header.station,
        location: header.location,
        channel: header.channel,
        starttime: header.starttime,
        samples: samples_f64,
        sampling_rate,
    })
}

pub fn parse_mseed_record(data: &[u8]) -> Result<Vec<TraceSegment>, Box<dyn std::error::Error>> {
    let mut segments = Vec::new();
    let mut offset = 0;

    // MiniSEED records are typically 512 bytes.
    // We iterate through chunks of 512 bytes.
    while offset + 512 <= data.len() {
        let record = &data[offset..offset + 512];

        match parse_header(record) {
            Ok(header) => {
                let data_start = header.data_offset as usize;

                if data_start < 512 {
                    let compressed_data = &record[data_start..512];

                    let result = match header.encoding {
                        10 => SteimDecoder::decode_steim1(
                            compressed_data,
                            header.num_samples as usize,
                        ),
                        11 => SteimDecoder::decode_steim2(
                            compressed_data,
                            header.num_samples as usize,
                        ),
                        _ => Err(crate::parser::steim::SteimError::InvalidSteimCode(
                            header.encoding,
                        )),
                    };

                    match result {
                        Ok(samples) => {
                            let samples_f64: Vec<f64> = samples.iter().map(|&x| x as f64).collect();

                            let sampling_rate = if header.sample_rate_factor > 0 {
                                if header.sample_rate_multiplier > 0 {
                                    header.sample_rate_factor as f64
                                        * header.sample_rate_multiplier as f64
                                } else {
                                    header.sample_rate_factor as f64
                                        / (-header.sample_rate_multiplier as f64)
                                }
                            } else {
                                100.0 // Default for Raspberry Shake
                            };

                            segments.push(TraceSegment {
                                network: header.network,
                                station: header.station,
                                location: header.location,
                                channel: header.channel,
                                starttime: header.starttime,
                                samples: samples_f64,
                                sampling_rate,
                            });
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Steim decode error in record {}: {}",
                                header.sequence_number,
                                e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                tracing::debug!("Header parse error at offset {}: {}", offset, e);
            }
        }
        offset += 512;
    }

    Ok(segments)
}

pub fn parse_mseed_file(path: &str) -> Result<Vec<TraceSegment>, Box<dyn std::error::Error>> {
    let data = std::fs::read(path)?;
    parse_mseed_record(&data)
}
