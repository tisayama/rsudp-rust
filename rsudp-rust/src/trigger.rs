use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertEventType {
    Trigger,
    Reset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub timestamp: DateTime<Utc>,
    pub channel: String,
    pub event_type: AlertEventType,
    pub ratio: f64,
    pub message: String,
}

impl std::fmt::Display for AlertEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] Channel {}: {}",
            self.timestamp, self.channel, self.message
        )
    }
}

#[derive(Debug, Clone)]
pub struct TriggerConfig {
    pub sta_sec: f64,
    pub lta_sec: f64,
    pub threshold: f64,
    pub reset_threshold: f64,
    pub highpass: f64,
    pub lowpass: f64,
    pub target_channel: String, // e.g. "EHZ" or "HZ" (suffix match)
}

struct Biquad {
    b0: f64, b1: f64, b2: f64,
    a1: f64, a2: f64,
    x1: f64, x2: f64,
    y1: f64, y2: f64,
}

impl Biquad {
    fn new(b0: f64, b1: f64, b2: f64, a1: f64, a2: f64) -> Self {
        Self { b0, b1, b2, a1, a2, x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0 }
    }

    fn process(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
                - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

struct BandpassFilter {
    sections: Vec<Biquad>,
    initialized: bool,
}

impl BandpassFilter {
    fn new_100hz_01_20() -> Self {
        // Butterworth 2nd order (4th order cascaded) 0.1-2.0Hz @ 100Hz
        let s1 = Biquad::new(0.00362168, 0.00724336, 0.00362168, -1.91119707, 0.91497583);
        let s2 = Biquad::new(1.0, -2.0, 1.0, -1.9911143, 0.9911536);
        Self { sections: vec![s1, s2], initialized: false }
    }

    fn reset(&mut self) {
        self.initialized = false;
        for s in &mut self.sections {
            s.x1 = 0.0; s.x2 = 0.0;
            s.y1 = 0.0; s.y2 = 0.0;
        }
    }

    fn process(&mut self, sample: f64) -> f64 {
        // Simple initialization to avoid large step response from 0 to first sample
        if !self.initialized {
            for s in &mut self.sections {
                s.x1 = sample;
                s.x2 = sample;
                s.y1 = sample;
                s.y2 = sample;
            }
            self.initialized = true;
        }

        let mut x = sample;
        for s in &mut self.sections {
            x = s.process(x);
        }
        x
    }
}

pub struct TriggerManager {
    config: TriggerConfig,
    states: HashMap<String, StaLtaState>,
}

struct StaLtaState {
    sta: f64,
    lta: f64,
    sample_count: u64,
    triggered: bool,
    max_ratio: f64,
    filter: BandpassFilter,
    last_timestamp: Option<DateTime<Utc>>,
}

impl TriggerManager {
    pub fn new(config: TriggerConfig) -> Self {
        Self {
            config,
            states: HashMap::new(),
        }
    }

    pub fn add_sample(
        &mut self,
        id: &str,
        sample: f64,
        timestamp: DateTime<Utc>,
        sensitivity: f64,
    ) -> Option<AlertEvent> {
        // Filter by target channel (simple contains check)
        if !id.contains(&self.config.target_channel) {
            return None;
        }

        let state = self.states.entry(id.to_string()).or_insert_with(|| StaLtaState {
            sta: 0.0,
            lta: 0.0,
            sample_count: 0,
            triggered: false,
            max_ratio: 0.0,
            filter: BandpassFilter::new_100hz_01_20(),
            last_timestamp: None,
        });

        // Detect jump
        if let Some(last_ts) = state.last_timestamp {
            let expected = last_ts + Duration::milliseconds(10); // Assume 100Hz
            let diff = (timestamp - expected).num_milliseconds().abs();
            if diff > 1000 {
                info!("Temporal jump detected on channel {} ({:?} -> {:?}). Resetting STA/LTA.", id, last_ts, timestamp);
                state.sta = 0.0;
                state.lta = 0.0;
                state.sample_count = 0;
                state.triggered = false;
                state.max_ratio = 0.0;
                state.filter.reset();
            }
        }
        state.last_timestamp = Some(timestamp);

        // 1. Convert to Physical Units (Deconvolution)
        // If sensitivity is 0 or invalid, use raw counts to avoid div/0
        let phys_val = if sensitivity > 0.0 { sample / sensitivity } else { sample };

        // 2. Apply Bandpass Filter
        // Filter is applied to physical values now
        let filtered_val = state.filter.process(phys_val);
        let val = filtered_val.abs();

        state.sample_count += 1;

        // 3. Update STA/LTA
        let sta_alpha = 1.0 / (self.config.sta_sec * 100.0);
        let lta_alpha = 1.0 / (self.config.lta_sec * 100.0);

        state.sta = (1.0 - sta_alpha) * state.sta + sta_alpha * val;
        state.lta = (1.0 - lta_alpha) * state.lta + lta_alpha * val;

        // 4. Warm-up Check (Wait for LTA window)
        let lta_samples = (self.config.lta_sec * 100.0) as u64;
        if state.sample_count < lta_samples {
            return None;
        }

        // Avoid division by zero
        let ratio = if state.lta > 1e-20 {
            state.sta / state.lta
        } else {
            0.0
        };

        if !state.triggered && ratio > self.config.threshold {
            state.triggered = true;
            state.max_ratio = ratio;
            let message = format!(
                "Trigger threshold {} exceeded (ratio: {:.4}). ALARM!",
                self.config.threshold, ratio
            );
            return Some(AlertEvent {
                timestamp,
                channel: id.to_string(),
                event_type: AlertEventType::Trigger,
                ratio,
                message,
            });
        } else if state.triggered {
            state.max_ratio = state.max_ratio.max(ratio);
            if ratio < self.config.reset_threshold {
                state.triggered = false;
                let message = format!(
                    "Ratio {:.4} fell below reset threshold {}. RESET. Max ratio: {:.4}",
                    ratio, self.config.reset_threshold, state.max_ratio
                );
                let ev = AlertEvent {
                    timestamp,
                    channel: id.to_string(),
                    event_type: AlertEventType::Reset,
                    ratio,
                    message,
                };
                return Some(ev);
            }
        }

                None

            }

        }

        

        #[cfg(test)]

        mod tests {

            use super::*;

        

            #[test]

            fn test_filter_state_continuity() {

                let mut filter_continuous = BandpassFilter::new_100hz_01_20();

                let mut filter_chunked = BandpassFilter::new_100hz_01_20();

        

                // Simulate 100 samples

                let samples: Vec<f64> = (0..100).map(|i| (i as f64).sin()).collect();

        

                // 1. Process continuous

                let mut continuous_output = Vec::new();

                for &s in &samples {

                    continuous_output.push(filter_continuous.process(s));

                }

        

                // 2. Process chunked (simulating packet reset if any)

                // Note: BandpassFilter doesn't have a reset method called implicitly,

                // but if TriggerManager creates a NEW filter or resets it wrongly, we'd see it.

                // Here we test the filter logic itself first.

                let mut chunked_output = Vec::new();

                for chunk in samples.chunks(25) {

                    // Simulate potential state loss if logic was wrong (it's not here, but verifying)

                    for &s in chunk {

                        chunked_output.push(filter_chunked.process(s));

                    }

                }

        

                // Verify outputs match exactly

                for (i, (c, ch)) in continuous_output.iter().zip(chunked_output.iter()).enumerate() {

                    assert!((c - ch).abs() < 1e-9, "Mismatch at sample {}: continuous={}, chunked={}", i, c, ch);

                }

            }

            

            #[test]

            fn test_trigger_manager_chunking() {

                let config = TriggerConfig {

                    sta_sec: 1.0,

                    lta_sec: 10.0,

                    threshold: 3.0,

                    reset_threshold: 1.5,

                    highpass: 0.1,

                    lowpass: 5.0,

                    target_channel: "HZ".to_string(),

                };

                

                let mut tm_continuous = TriggerManager::new(config.clone());

                let mut tm_chunked = TriggerManager::new(config);

                

                let id = "TEST.HZ";

                let start_time = Utc::now();

                let samples: Vec<f64> = (0..500).map(|i| if i > 300 && i < 350 { 100.0 } else { 1.0 }).collect(); // Impulse at 300

                

                // 1. Continuous feed (1 sample at a time loop, standard)

                let mut continuous_ratios = Vec::new();

                for (i, &s) in samples.iter().enumerate() {

                    let ts = start_time + Duration::milliseconds(i as i64 * 10);

                    tm_continuous.add_sample(id, s, ts, 1.0);

                    if let Some(state) = tm_continuous.states.get(id) {

                        if state.lta > 0.0 {

                            continuous_ratios.push(state.sta / state.lta);

                        } else {

                            continuous_ratios.push(0.0);

                        }

                    }

                }

                

                // 2. Chunked feed (25 samples)

                let mut chunked_ratios = Vec::new();

                for (chunk_idx, chunk) in samples.chunks(25).enumerate() {

                    for (i, &s) in chunk.iter().enumerate() {

                        let absolute_idx = chunk_idx * 25 + i;

                        let ts = start_time + Duration::milliseconds(absolute_idx as i64 * 10);

                        tm_chunked.add_sample(id, s, ts, 1.0);

                        

                        if let Some(state) = tm_chunked.states.get(id) {

                            if state.lta > 0.0 {

                                chunked_ratios.push(state.sta / state.lta);

                            } else {

                                chunked_ratios.push(0.0);

                            }

                        }

                    }

                }

                

                // Compare ratios

                assert_eq!(continuous_ratios.len(), chunked_ratios.len());

                        for (i, (c, ch)) in continuous_ratios.iter().zip(chunked_ratios.iter()).enumerate() {

                            assert!((c - ch).abs() < 1e-9, "Ratio mismatch at sample {}: cont={}, chunked={}", i, c, ch);

                        }

                    }

                

                    #[test]

                    fn test_temporal_jump_tolerance() {

                        let config = TriggerConfig {

                            sta_sec: 1.0,

                            lta_sec: 10.0,

                            threshold: 3.0,

                            reset_threshold: 1.5,

                            highpass: 0.1,

                            lowpass: 5.0,

                            target_channel: "HZ".to_string(),

                        };

                        let mut tm = TriggerManager::new(config);

                        let id = "TEST.HZ";

                        let start_time = Utc::now();

                

                        // 1. Process initial 100 samples

                        for i in 0..100 {

                            let ts = start_time + Duration::milliseconds(i * 10);

                            tm.add_sample(id, 1.0, ts, 1.0);

                        }

                        

                        let state_before = tm.states.get(id).unwrap();

                        assert_eq!(state_before.sample_count, 100);

                

                        // 2. Introduce small jitter (e.g., 20ms gap instead of 10ms), well below 1000ms threshold

                        // Expected next is +10ms, but we give +20ms. Diff = 10ms.

                        let next_ts = start_time + Duration::milliseconds(100 * 10 + 10);

                        

                        tm.add_sample(id, 1.0, next_ts, 1.0);

                        

                        let state_after = tm.states.get(id).unwrap();

                        // Should NOT have reset

                        assert_eq!(state_after.sample_count, 101);

                        

                        // 3. Large jump (reset condition)

                        let jump_ts = next_ts + Duration::milliseconds(2000);

                        tm.add_sample(id, 1.0, jump_ts, 1.0);

                        

                        let state_reset = tm.states.get(id).unwrap();

                        // Should have reset

                        assert_eq!(state_reset.sample_count, 1);

                    }

                }

                

        