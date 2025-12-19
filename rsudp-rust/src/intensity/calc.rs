use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::intensity::{IntensityConfig, IntensityResult, get_shindo_class, filter::JmaFilter};

pub struct IntensityManager {
    config: IntensityConfig,
    buffers: HashMap<String, Vec<f64>>,
    filter: JmaFilter,
    results: Vec<IntensityResult>,
}

impl IntensityManager {
    pub fn new(config: IntensityConfig) -> Self {
        let buffers = config.channels.iter().map(|c| (c.clone(), Vec::new())).collect();
        Self {
            config,
            buffers,
            filter: JmaFilter::new(100.0),
            results: Vec::new(),
        }
    }

    pub fn config(&self) -> &IntensityConfig {
        &self.config
    }

    pub fn add_samples(&mut self, samples_map: HashMap<String, Vec<f64>>, start_time: DateTime<Utc>) {
        for (ch, data) in samples_map {
            if let Some(buf) = self.buffers.get_mut(&ch) {
                buf.extend(data);
            }
        }

        // Processing 60s window (6000 samples at 100Hz)
        let window_len = (self.config.sample_rate * 60.0) as usize;
        
        // Calculate if we have enough data
        let mut can_calculate = true;
        for ch in &self.config.channels {
            if self.buffers.get(ch).unwrap().len() < window_len {
                can_calculate = false;
                break;
            }
        }

        if can_calculate {
            // SLIDING WINDOW: We take the LAST 60 seconds
            let mut window_data = [Vec::new(), Vec::new(), Vec::new()];
            for (i, ch) in self.config.channels.iter().enumerate().take(3) {
                let buf = self.buffers.get(ch).unwrap();
                let start_idx = buf.len() - window_len;
                window_data[i] = buf[start_idx..].to_vec();
            }

            // Convert Counts -> Gal
            for i in 0..3 {
                let sens = self.config.sensitivities.get(i).unwrap_or(&1.0);
                for val in &mut window_data[i] {
                    *val *= 100.0 * sens; // Counts -> m/s^2 -> Gal
                }
            }

            let intensity = self.filter.calculate_intensity(&window_data[0], &window_data[1], &window_data[2]);
            let shindo_class = get_shindo_class(intensity);

            self.results.push(IntensityResult {
                timestamp: start_time,
                intensity,
                shindo_class,
            });

            // Maintain buffer size: remove samples older than 60s + some margin
            // We keep up to 70s to ensure we don't recalculate too often or lose data
            let max_buf = (self.config.sample_rate * 70.0) as usize;
            for ch in &self.config.channels {
                let buf = self.buffers.get_mut(ch).unwrap();
                if buf.len() > max_buf {
                    buf.drain(0..(buf.len() - window_len));
                }
            }
        }
    }

    pub fn get_results(&mut self) -> Vec<IntensityResult> {
        std::mem::take(&mut self.results)
    }
}