use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

mod calc;
pub mod filter;

pub use calc::IntensityManager;

#[derive(Debug, Clone)]
pub struct IntensityConfig {
    pub channels: Vec<String>,
    pub sample_rate: f64,
    pub sensitivities: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntensityResult {
    pub timestamp: DateTime<Utc>,
    pub intensity: f64,
    pub shindo_class: String,
}

pub fn get_shindo_class(intensity: f64) -> String {
    if intensity < 0.5 { "0".to_string() }
    else if intensity < 1.5 { "1".to_string() }
    else if intensity < 2.5 { "2".to_string() }
    else if intensity < 3.5 { "3".to_string() }
    else if intensity < 4.5 { "4".to_string() }
    else if intensity < 5.0 { "5-".to_string() }
    else if intensity < 5.5 { "5+".to_string() }
    else if intensity < 6.0 { "6-".to_string() }
    else if intensity < 6.5 { "6+".to_string() }
    else { "7".to_string() }
}
