use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use crate::intensity::{IntensityConfig, IntensityResult, get_shindo_class, filter::JmaFilter};

pub struct IntensityManager {
    config: IntensityConfig,
    buffers: HashMap<String, Vec<f64>>,
    buffer_start_times: HashMap<String, DateTime<Utc>>,
    filter: JmaFilter,
    results: Vec<IntensityResult>,
}

impl IntensityManager {
    pub fn new(config: IntensityConfig) -> Self {
        let buffers = config.channels.iter().map(|c| (c.clone(), Vec::new())).collect();
        let buffer_start_times = HashMap::new();
        Self {
            config,
            buffers,
            buffer_start_times,
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
                if buf.is_empty() {
                    self.buffer_start_times.insert(ch.clone(), start_time);
                }
                buf.extend(data);
            }
        }

        let window_len = (self.config.sample_rate * 60.0) as usize;
        
        // 1. Check if we have enough data in all buffers
        if self.config.channels.iter().any(|ch| self.buffers.get(ch).unwrap().len() < window_len) {
            return;
        }

        // 2. Find the latest start time among buffers to align
        let mut latest_start = DateTime::<Utc>::MIN_UTC;
        for ch in &self.config.channels {
            if let Some(&st) = self.buffer_start_times.get(ch) {
                if st > latest_start { latest_start = st; }
            }
        }

        // 3. Drop samples that are too old to align all buffers to the same start time
        for ch in &self.config.channels {
            let st = *self.buffer_start_times.get(ch).unwrap();
            let diff = (latest_start - st).num_milliseconds();
            if diff > 0 {
                let samples_to_drop = (diff as f64 * self.config.sample_rate / 1000.0).round() as usize;
                let buf = self.buffers.get_mut(ch).unwrap();
                if samples_to_drop < buf.len() {
                    buf.drain(0..samples_to_drop);
                    let new_st = st + Duration::milliseconds((samples_to_drop as f64 * 1000.0 / self.config.sample_rate).round() as i64);
                    self.buffer_start_times.insert(ch.clone(), new_st);
                }
            }
        }

        // 4. Calculate intensity if we still have enough data
        let mut can_calculate = true;
        for ch in &self.config.channels {
            if self.buffers.get(ch).unwrap().len() < window_len {
                can_calculate = false;
                break;
            }
        }

        if can_calculate {
            let mut window_data = [Vec::new(), Vec::new(), Vec::new()];
            for (i, ch) in self.config.channels.iter().enumerate().take(3) {
                let buf = self.buffers.get(ch).unwrap();
                // Take the window starting from the latest common start time
                window_data[i] = buf[0..window_len].to_vec();
            }

            // Convert Counts -> Gal
            for i in 0..3 {
                let sens = self.config.sensitivities.get(i).unwrap_or(&1.0);
                for val in &mut window_data[i] {
                    *val *= 100.0 * sens;
                }
            }

            let intensity = self.filter.calculate_intensity(&window_data[0], &window_data[1], &window_data[2]);
            let shindo_class = get_shindo_class(intensity);

            self.results.push(IntensityResult {
                timestamp: latest_start + Duration::seconds(60), // Window end time
                intensity,
                shindo_class,
            });

            // SLIDING WINDOW: Remove oldest samples (e.g. 1 second worth)
            let slide_samples = (self.config.sample_rate * 1.0) as usize;
            for ch in &self.config.channels {
                let buf = self.buffers.get_mut(ch).unwrap();
                let st = self.buffer_start_times.get_mut(ch).unwrap();
                buf.drain(0..slide_samples);
                *st = *st + Duration::milliseconds((slide_samples as f64 * 1000.0 / self.config.sample_rate).round() as i64);
            }
        }
    }

    pub fn get_results(&mut self) -> Vec<IntensityResult> {
        std::mem::take(&mut self.results)
    }
}