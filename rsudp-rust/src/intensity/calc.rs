use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use crate::intensity::{IntensityConfig, IntensityResult, get_shindo_class, filter::JmaFilter};
use tracing::info;

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

    pub fn reset(&mut self) {
        for buf in self.buffers.values_mut() {
            buf.clear();
        }
        self.buffer_start_times.clear();
    }

    pub fn add_samples(&mut self, samples_map: HashMap<String, Vec<f64>>, start_time: DateTime<Utc>) {
        let mut needs_reset = false;
        for (ch, _) in &samples_map {
            if let Some(buf) = self.buffers.get(ch) {
                if !buf.is_empty() {
                    if let Some(&st) = self.buffer_start_times.get(ch) {
                        let expected_next = st + Duration::milliseconds((buf.len() as f64 * 1000.0 / self.config.sample_rate).round() as i64);
                        let diff_ms = (start_time - expected_next).num_milliseconds();
                        
                        if diff_ms.abs() > 10000 { // 10 seconds threshold for hard reset
                            info!("Temporal jump detected on {} ({:?} -> {:?}, diff: {}ms). Resetting intensity buffers.", ch, expected_next, start_time, diff_ms);
                            needs_reset = true;
                            break;
                        }
                    }
                }
            }
        }

        if needs_reset {
            self.reset();
        }

        for (ch, data) in samples_map {
            if let Some(buf) = self.buffers.get_mut(&ch) {
                if buf.is_empty() {
                    self.buffer_start_times.insert(ch.clone(), start_time);
                    buf.extend(data);
                } else {
                    let st = *self.buffer_start_times.get(&ch).unwrap();
                    let expected_next = st + Duration::milliseconds((buf.len() as f64 * 1000.0 / self.config.sample_rate).round() as i64);
                    let diff_ms = (start_time - expected_next).num_milliseconds();
                    let sample_interval_ms = 1000.0 / self.config.sample_rate;

                    if diff_ms.abs() < 100 { // Ignore small jitter (< 10 samples)
                        buf.extend(data);
                    } else if diff_ms > 0 {
                        // Gap: pad with last value but with a slight fade to zero (mean) if it's a long gap
                        let pad_samples = (diff_ms as f64 / sample_interval_ms).round() as usize;
                        if pad_samples > 0 {
                            let last_val = buf.last().cloned().unwrap_or(0.0);
                            for i in 0..pad_samples {
                                // Linear fade over 1 second (100 samples)
                                let factor = if i < 100 { 1.0 - (i as f64 / 100.0) } else { 0.0 };
                                buf.push(last_val * factor);
                            }
                        }
                        buf.extend(data);
                    } else {
                        // Overlap: Trim new data
                        let trim_samples = (diff_ms.abs() as f64 / sample_interval_ms).round() as usize;
                        if trim_samples < data.len() {
                            buf.extend(&data[trim_samples..]);
                        }
                    }
                }
            }
        }

        let window_len = (self.config.sample_rate * 60.0) as usize;
        
        if self.config.channels.iter().any(|ch| self.buffers.get(ch).map(|b| b.len()).unwrap_or(0) < window_len) {
            return;
        }

        let mut latest_start = DateTime::<Utc>::MIN_UTC;
        for ch in &self.config.channels {
            if let Some(&st) = self.buffer_start_times.get(ch) {
                if st > latest_start { latest_start = st; }
            }
        }

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
                } else {
                    buf.clear();
                    self.buffer_start_times.insert(ch.clone(), latest_start);
                }
            }
        }

        while self.config.channels.iter().all(|ch| self.buffers.get(ch).unwrap().len() >= window_len) {
            let mut window_data = [Vec::new(), Vec::new(), Vec::new()];
            for (i, ch) in self.config.channels.iter().enumerate().take(3) {
                let buf = self.buffers.get(ch).unwrap();
                window_data[i] = buf[0..window_len].to_vec();
            }

            for i in 0..3 {
                let sens = self.config.sensitivities.get(i).unwrap_or(&1.0);
                for val in &mut window_data[i] {
                    *val *= 100.0 * sens;
                }
            }

            let intensity = self.filter.calculate_intensity(&window_data[0], &window_data[1], &window_data[2]);
            let shindo_class = get_shindo_class(intensity);

            self.results.push(IntensityResult {
                timestamp: latest_start + Duration::seconds(60),
                intensity,
                shindo_class,
            });

            let slide_samples = (self.config.sample_rate * 1.0) as usize;
            for ch in &self.config.channels {
                let buf = self.buffers.get_mut(ch).unwrap();
                let st = self.buffer_start_times.get_mut(ch).unwrap();
                buf.drain(0..slide_samples);
                *st = *st + Duration::milliseconds((slide_samples as f64 * 1000.0 / self.config.sample_rate).round() as i64);
            }
            latest_start = latest_start + Duration::milliseconds((slide_samples as f64 * 1000.0 / self.config.sample_rate).round() as i64);
        }
    }

    pub fn get_results(&mut self) -> Vec<IntensityResult> {
        std::mem::take(&mut self.results)
    }
}