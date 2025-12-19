pub mod routes;
pub mod stream;
pub mod test_utils;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
pub use crate::trigger::{AlertEvent, AlertEventType};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformPacket {
    pub channel_id: String,
    pub timestamp: DateTime<Utc>,
    pub samples: Vec<f32>,
    pub sample_rate: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlotSettings {
    pub active_channels: Vec<String>,
    pub window_seconds: u32,
    pub auto_scale: bool,
    pub theme: String,
}

impl WaveformPacket {
    pub fn to_binary(&self) -> Vec<u8> {
        use byteorder::{LittleEndian, WriteBytesExt};
        let mut buf = Vec::new();
        // Type: 0 for Waveform
        buf.push(0);
        // Channel ID length and string
        buf.push(self.channel_id.len() as u8);
        buf.extend_from_slice(self.channel_id.as_bytes());
        // Timestamp (Unix timestamp micros)
        let ts = self.timestamp.timestamp_micros();
        buf.write_i64::<LittleEndian>(ts).unwrap();
        // Sample Rate
        buf.write_f32::<LittleEndian>(self.sample_rate).unwrap();
        // Samples count
        buf.write_u32::<LittleEndian>(self.samples.len() as u32).unwrap();
        // Samples
        for &s in &self.samples {
            buf.write_f32::<LittleEndian>(s).unwrap();
        }
        buf
    }
}