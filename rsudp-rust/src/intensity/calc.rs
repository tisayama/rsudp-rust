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
        // 1. Check for resets based on the *first* available channel state to decide global reset
        // This is a heuristic; if one channel jumps significantly, it's likely a stream reset.
        let mut needs_reset = false;
        for (ch, _) in &samples_map {
            if let Some(buf) = self.buffers.get(ch) {
                if !buf.is_empty() {
                    if let Some(&st) = self.buffer_start_times.get(ch) {
                        let expected_next = st + Duration::milliseconds((buf.len() as f64 * 1000.0 / self.config.sample_rate).round() as i64);
                        let diff_ms = (start_time - expected_next).num_milliseconds();
                        
                        // If gap/jump is more than 1 second (positive or negative), reset all buffers
                        if diff_ms.abs() > 1000 {
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

        // 2. Process each channel with strict alignment
        for (ch, data) in samples_map {
            if let Some(buf) = self.buffers.get_mut(&ch) {
                if buf.is_empty() {
                    self.buffer_start_times.insert(ch.clone(), start_time);
                    buf.extend(data);
                } else {
                    // Calculate expected next sample time
                    let st = *self.buffer_start_times.get(&ch).unwrap();
                    let expected_next = st + Duration::milliseconds((buf.len() as f64 * 1000.0 / self.config.sample_rate).round() as i64);
                    let diff_ms = (start_time - expected_next).num_milliseconds();
                    let sample_interval_ms = 1000.0 / self.config.sample_rate;

                    if diff_ms.abs() as f64 <= sample_interval_ms * 0.5 {
                        // Continuous (within half a sample margin)
                        buf.extend(data);
                    } else if diff_ms > 0 {
                        // Gap: Pad with zeros
                        let pad_samples = (diff_ms as f64 / sample_interval_ms).round() as usize;
                        if pad_samples > 0 {
                            // info!("Padding {} samples ({}ms) on {}", pad_samples, diff_ms, ch);
                            buf.extend(std::iter::repeat(0.0).take(pad_samples));
                        }
                        buf.extend(data);
                    } else {
                        // Overlap: Trim new data
                        let trim_samples = (diff_ms.abs() as f64 / sample_interval_ms).round() as usize;
                        if trim_samples < data.len() {
                            // info!("Trimming {} samples ({}ms) from head of {} due to overlap", trim_samples, diff_ms, ch);
                            buf.extend(&data[trim_samples..]);
                        } else {
                            // New data is completely overlapped/old, ignore it
                        }
                    }
                }
            }
        }

        let window_len = (self.config.sample_rate * 60.0) as usize;
        
        // 1. Check if we have enough data in all buffers
        if self.config.channels.iter().any(|ch| self.buffers.get(ch).map(|b| b.len()).unwrap_or(0) < window_len) {
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
                } else {
                    // If we drop everything, reset start time to current latest
                    buf.clear();
                    self.buffer_start_times.insert(ch.clone(), latest_start);
                }
            }
        }

        // 4. Calculate intensity if we still have enough data
        while self.config.channels.iter().all(|ch| self.buffers.get(ch).unwrap().len() >= window_len) {
            let mut window_data = [Vec::new(), Vec::new(), Vec::new()];
            for (i, ch) in self.config.channels.iter().enumerate().take(3) {
                let buf = self.buffers.get(ch).unwrap();
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
            // Advance latest_start for the next loop iteration
            latest_start = latest_start + Duration::milliseconds((slide_samples as f64 * 1000.0 / self.config.sample_rate).round() as i64);
        }
    }

    pub fn get_results(&mut self) -> Vec<IntensityResult> {
        std::mem::take(&mut self.results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_manager() -> IntensityManager {
        IntensityManager::new(IntensityConfig {
            channels: vec!["ENE".to_string(), "ENN".to_string(), "ENZ".to_string()],
            sample_rate: 100.0,
            sensitivities: vec![1.0, 1.0, 1.0],
        })
    }

    #[test]
    fn test_temporal_jump_reset() {
        let mut im = create_manager();
        let start_time = Utc::now();
        
        let mut map = HashMap::new();
        map.insert("ENE".to_string(), vec![0.0; 100]);
        im.add_samples(map.clone(), start_time);
        assert_eq!(im.buffers.get("ENE").unwrap().len(), 100);

        // Jump back 1 hour
        let jump_time = start_time - Duration::hours(1);
        im.add_samples(map, jump_time);
        
        // Should have reset and added the new 100 samples
        assert_eq!(im.buffers.get("ENE").unwrap().len(), 100);
        assert_eq!(*im.buffer_start_times.get("ENE").unwrap(), jump_time);
    }

    #[test]
    fn test_buffer_alignment() {
        let mut im = create_manager();
        let start_time = Utc::now();
        
        // ENE starts at t
        let mut map1 = HashMap::new();
        map1.insert("ENE".to_string(), vec![0.0; 100]);
        im.add_samples(map1, start_time);

        // ENN/ENZ start at t+1s
        let mut map2 = HashMap::new();
        let start_time_plus_1 = start_time + Duration::seconds(1);
        map2.insert("ENN".to_string(), vec![0.0; 6000]); // Full window
        map2.insert("ENZ".to_string(), vec![0.0; 6000]);
        im.add_samples(map2, start_time_plus_1);

        // Now ENE gets more data to finish window
        let mut map3 = HashMap::new();
        map3.insert("ENE".to_string(), vec![0.0; 6000]);
        // start_time of this packet is t + 1s (100 samples later)
        im.add_samples(map3, start_time + Duration::seconds(1));

        // After alignment, all buffers should start at start_time_plus_1
        // BUT calculation will run once and slide by 1s, so they will be at start_time_plus_2
        let start_time_plus_2 = start_time + Duration::seconds(2);
        assert_eq!(*im.buffer_start_times.get("ENE").unwrap(), start_time_plus_2);
        assert_eq!(im.buffers.get("ENE").unwrap().len(), 5900); // 6000 - 100
    }
}
