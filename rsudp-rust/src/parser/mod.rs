use chrono::{DateTime, Utc};

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
