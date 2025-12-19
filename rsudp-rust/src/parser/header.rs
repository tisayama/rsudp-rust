use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read};
use chrono::{DateTime, Utc, TimeZone};

#[derive(Debug, Clone)]
pub struct MSeedHeader {
    pub sequence_number: String,
    pub station: String,
    pub location: String,
    pub channel: String,
    pub network: String,
    pub starttime: DateTime<Utc>,
    pub num_samples: u16,
    pub sample_rate_factor: i16,
    pub sample_rate_multiplier: i16,
    pub encoding: u8,
    pub byte_order: u8,
    pub data_offset: u16,
}

pub fn parse_header(data: &[u8]) -> Result<MSeedHeader, Box<dyn std::error::Error>> {
    let mut rdr = Cursor::new(data);
    
    let mut seq_bytes = [0u8; 6];
    rdr.read_exact(&mut seq_bytes)?;
    let sequence_number = String::from_utf8_lossy(&seq_bytes).to_string();
    
    let _indicator = rdr.read_u8()?;
    let _reserved = rdr.read_u8()?;
    
    let mut station_bytes = [0u8; 5];
    rdr.read_exact(&mut station_bytes)?;
    let station = String::from_utf8_lossy(&station_bytes).trim().to_string();
    
    let mut loc_bytes = [0u8; 2];
    rdr.read_exact(&mut loc_bytes)?;
    let location = String::from_utf8_lossy(&loc_bytes).trim().to_string();
    
    let mut chan_bytes = [0u8; 3];
    rdr.read_exact(&mut chan_bytes)?;
    let channel = String::from_utf8_lossy(&chan_bytes).trim().to_string();
    
    let mut net_bytes = [0u8; 2];
    rdr.read_exact(&mut net_bytes)?;
    let network = String::from_utf8_lossy(&net_bytes).trim().to_string();
    
    let year = rdr.read_u16::<BigEndian>()?;
    let day = rdr.read_u16::<BigEndian>()?;
    let hour = rdr.read_u8()?;
    let minute = rdr.read_u8()?;
    let second = rdr.read_u8()?;
    let _unused = rdr.read_u8()?;
    let microsecond = rdr.read_u16::<BigEndian>()? as u32 * 100;
    
    let starttime = Utc.with_ymd_and_hms(year as i32, 1, 1, hour as u32, minute as u32, second as u32)
        .unwrap() + chrono::Duration::days(day as i64 - 1) + chrono::Duration::microseconds(microsecond as i64);
        
    let num_samples = rdr.read_u16::<BigEndian>()?;
    let sample_rate_factor = rdr.read_i16::<BigEndian>()?;
    let sample_rate_multiplier = rdr.read_i16::<BigEndian>()?;
    
    let _act_flags = rdr.read_u8()?;
    let _io_flags = rdr.read_u8()?;
    let _dq_flags = rdr.read_u8()?;
    let _num_blockettes = rdr.read_u8()?;
    let _time_correction = rdr.read_i32::<BigEndian>()?;
    let data_offset = rdr.read_u16::<BigEndian>()?;
    let blockette_offset = rdr.read_u16::<BigEndian>()?;
    
    let mut encoding = 11;
    let mut byte_order = 1;
    
    let mut current_b_offset = blockette_offset;
    while current_b_offset >= 48 && (current_b_offset as usize) < data.len() {
        let mut b_rdr = Cursor::new(&data[current_b_offset as usize..]);
        let b_type = b_rdr.read_u16::<BigEndian>()?;
        let next_b_offset = b_rdr.read_u16::<BigEndian>()?;
        
        if b_type == 1000 {
            encoding = b_rdr.read_u8()?;
            byte_order = b_rdr.read_u8()?;
            // Blockette 1000 is 8 bytes, next 2 bytes are reserved or data length
            break;
        }
        
        if next_b_offset == 0 || next_b_offset == current_b_offset { break; }
        current_b_offset = next_b_offset;
    }

    Ok(MSeedHeader {
        sequence_number,
        station,
        location,
        channel,
        network,
        starttime,
        num_samples,
        sample_rate_factor,
        sample_rate_multiplier,
        encoding,
        byte_order,
        data_offset,
    })
}
