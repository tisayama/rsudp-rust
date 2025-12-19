use byteorder::{BigEndian, ReadBytesExt};
use chrono::{DateTime, Utc, TimeZone};
use std::io::{Cursor, Read};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HeaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid BTIME: {0}")]
    InvalidBTime(String),
}

#[derive(Debug, Clone)]
pub struct SeedHeader {
    pub station: String,
    pub location: String,
    pub channel: String,
    pub network: String,
    pub start_time: DateTime<Utc>,
    pub num_samples: u16,
    pub sample_rate_factor: i16,
    pub sample_rate_multiplier: i16,
    pub data_offset: u16,
    pub encoding: u8,
    pub record_size: usize,
}

impl SeedHeader {
    pub fn parse(data: &[u8]) -> Result<Self, HeaderError> {
        let mut rdr = Cursor::new(data);

        let mut _seq = [0u8; 6];
        rdr.read_exact(&mut _seq)?;
        let _indicator = rdr.read_u8()?;
        let _reserved = rdr.read_u8()?;

        let mut station_bytes = [0u8; 5];
        rdr.read_exact(&mut station_bytes)?;
        let station = String::from_utf8_lossy(&station_bytes).trim().to_string();

        let mut location_bytes = [0u8; 2];
        rdr.read_exact(&mut location_bytes)?;
        let location = String::from_utf8_lossy(&location_bytes).trim().to_string();

        let mut channel_bytes = [0u8; 3];
        rdr.read_exact(&mut channel_bytes)?;
        let channel = String::from_utf8_lossy(&channel_bytes).trim().to_string();

        let mut network_bytes = [0u8; 2];
        rdr.read_exact(&mut network_bytes)?;
        let network = String::from_utf8_lossy(&network_bytes).trim().to_string();

        let year = rdr.read_u16::<BigEndian>()?;
        let day = rdr.read_u16::<BigEndian>()?;
        let hour = rdr.read_u8()?;
        let minute = rdr.read_u8()?;
        let second = rdr.read_u8()?;
        let _unused = rdr.read_u8()?;
        let microsecond_ticks = rdr.read_u16::<BigEndian>()?;

        let start_time = btime_to_datetime(year, day, hour, minute, second, microsecond_ticks)?;

        let num_samples = rdr.read_u16::<BigEndian>()?;
        let sample_rate_factor = rdr.read_i16::<BigEndian>()?;
        let sample_rate_multiplier = rdr.read_i16::<BigEndian>()?;

        let _activity_flags = rdr.read_u8()?;
        let _io_flags = rdr.read_u8()?;
        let _data_quality_flags = rdr.read_u8()?;
        let _num_blockettes = rdr.read_u8()?;
        let _time_correction = rdr.read_i32::<BigEndian>()?;
        let data_offset = rdr.read_u16::<BigEndian>()?;
        let blockette_offset = rdr.read_u16::<BigEndian>()?;

        let mut encoding = 11; 
        let mut record_size = 512;
        let mut current_offset = blockette_offset as usize;
        
        while current_offset > 0 && current_offset + 4 <= data.len() {
            let mut blkt_rdr = Cursor::new(&data[current_offset..]);
            let blkt_type = blkt_rdr.read_u16::<BigEndian>()?;
            let next_offset = blkt_rdr.read_u16::<BigEndian>()?;
            
            if blkt_type == 1000 {
                if current_offset + 7 <= data.len() {
                    encoding = data[current_offset + 4];
                    let rec_len_exp = data[current_offset + 6];
                    record_size = 1 << rec_len_exp;
                }
                break;
            }
            
            if next_offset == 0 {
                break;
            }
            current_offset = next_offset as usize;
        }

        Ok(Self {
            station,
            location,
            channel,
            network,
            start_time,
            num_samples,
            sample_rate_factor,
            sample_rate_multiplier,
            data_offset,
            encoding,
            record_size,
        })
    }

    pub fn sample_rate(&self) -> f64 {
        let factor = self.sample_rate_factor as f64;
        let multiplier = self.sample_rate_multiplier as f64;

        if factor > 0.0 && multiplier > 0.0 {
            factor * multiplier
        } else if factor > 0.0 && multiplier < 0.0 {
            -factor / multiplier
        } else if factor < 0.0 && multiplier > 0.0 {
            -multiplier / factor
        } else if factor < 0.0 && multiplier < 0.0 {
            1.0 / (factor * multiplier)
        } else {
            0.0
        }
    }
}

fn btime_to_datetime(
    year: u16,
    day: u16,
    hour: u8,
    minute: u8,
    second: u8,
    ticks: u16,
) -> Result<DateTime<Utc>, HeaderError> {
    let microseconds = (ticks as u32) * 100;
    let date = Utc.with_ymd_and_hms(year as i32, 1, 1, hour as u32, minute as u32, second as u32)
        .single()
        .ok_or_else(|| HeaderError::InvalidBTime("Base date invalid".into()))?;
    
    let date = date + chrono::Duration::days((day as i64) - 1);
    let date = date + chrono::Duration::microseconds(microseconds as i64);

    Ok(date)
}