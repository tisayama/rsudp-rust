use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        });

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
