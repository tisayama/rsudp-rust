use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertEventType {
    Trigger,
    Reset,
    Status,
}

#[derive(Debug, Clone)]
pub struct Biquad {
    pub b0: f64, pub b1: f64, pub b2: f64,
    pub a1: f64, pub a2: f64,
    pub s1: f64, pub s2: f64,
}

impl Biquad {
    pub fn new(b0: f64, b1: f64, b2: f64, a1: f64, a2: f64) -> Self {
        Self { b0, b1, b2, a1, a2, s1: 0.0, s2: 0.0 }
    }
    
    pub fn process(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.s1;
        self.s1 = self.b1 * x - self.a1 * y + self.s2;
        self.s2 = self.b2 * x - self.a2 * y;
        y
    }
}

pub fn butter_bandpass_sos(_order: usize, _low_freq: f64, _high_freq: f64, _fs: f64) -> Vec<Biquad> {
    vec![
        Biquad { b0: 1.091166705330671136e-05, b1: 2.182333410661342271e-05, b2: 1.091166705330671136e-05, a1: -1.799856289596911019e+00, a2: 8.118007230490338344e-01, s1:0.0, s2:0.0 },
        Biquad { b0: 1.000000000000000000e+00, b1: 2.000000000000000000e+00, b2: 1.000000000000000000e+00, a1: -1.902151139520082967e+00, a2: 9.168966689551941718e-01, s1:0.0, s2:0.0 },
        Biquad { b0: 1.000000000000000000e+00, b1: -2.000000000000000000e+00, b2: 1.000000000000000000e+00, a1: -1.987573235639373159e+00, a2: 9.876201932101479342e-01, s1:0.0, s2:0.0 },
        Biquad { b0: 1.000000000000000000e+00, b1: -2.000000000000000000e+00, b2: 1.000000000000000000e+00, a1: -1.995513812922410590e+00, a2: 9.955541951492401509e-01, s1:0.0, s2:0.0 },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub timestamp: DateTime<Utc>,
    pub channel: String,
    pub event_type: AlertEventType,
    pub ratio: f64,
    pub max_ratio: f64,
    pub message: String,
}

impl std::fmt::Display for AlertEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_str = match self.event_type {
            AlertEventType::Trigger => "ALARM",
            AlertEventType::Reset => "RESET",
            AlertEventType::Status => "STATUS",
        };
        f.write_str("[")?;
        f.write_str(&self.timestamp.to_rfc3339())?;
        f.write_str("] ")?;
        f.write_str(&self.channel)?;
        f.write_str(": ")?;
        f.write_str(type_str)?;
        
        if self.event_type == AlertEventType::Reset {
             // Use format macro directly with correct syntax
             write!(f, " (end ratio: {:.4}, max ratio: {:.4})", self.ratio, self.max_ratio)
        } else {
             write!(f, " (ratio: {:.4})", self.ratio)
        }
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
    pub target_channel: String,
    pub duration: f64,
}

pub struct TriggerManager {
    config: TriggerConfig,
    states: HashMap<String, StaLtaState>,
}

struct StaLtaState {
    triggered: bool,
    max_ratio: f64,
    last_timestamp: Option<DateTime<Utc>>,
    exceed_start: Option<DateTime<Utc>>,
    is_exceeding: bool,
    raw_buffer: VecDeque<f64>,
    sample_count: usize,
}

impl TriggerManager {
    pub fn new(config: TriggerConfig) -> Self {
        info!("TriggerManager initialized (Windowed STA/LTA Mode with Status).");
        Self { config, states: HashMap::new() }
    }

    pub fn add_sample(&mut self, id: &str, sample: f64, timestamp: DateTime<Utc>, _sensitivity: f64) -> Option<AlertEvent> {
        if !id.contains(&self.config.target_channel) { return None; }

        let clean_id = id.rsplit('.').next().unwrap_or(id).trim_matches('\'').trim().to_string();

        let highpass = self.config.highpass;
        let lowpass = self.config.lowpass;
        let nlta = (self.config.lta_sec * 100.0) as usize;
        let nsta = (self.config.sta_sec * 100.0) as usize;
        // Match Python rsudp window: nlta + one packet (25 samples at 100 SPS).
        // ObsPy's Stream.slice(endtime - lta_sec) yields nlta + ~packet_size samples
        // due to packet-boundary alignment, causing ndat > nlta which triggers
        // ObsPy's recursive_sta_lta to zero the first nlta output elements.
        let win_size = nlta + 25;
        let state = self.states.entry(clean_id.clone()).or_insert_with(|| StaLtaState {
            triggered: false, max_ratio: 0.0, last_timestamp: None, exceed_start: None, is_exceeding: false,
            raw_buffer: VecDeque::with_capacity(win_size),
            sample_count: 0,
        });

        // --- GAP DETECTION ---
        if let Some(last_ts) = state.last_timestamp {
            if (timestamp - last_ts).num_milliseconds().abs() > 1000 {
                state.raw_buffer.clear();
                state.sample_count = 0;
            }
        }
        state.last_timestamp = Some(timestamp);

        // --- WINDOWED STA/LTA (Python rsudp-faithful) ---
        // Store raw (unfiltered) sample in ring buffer
        state.raw_buffer.push_back(sample);
        if state.raw_buffer.len() > win_size {
            state.raw_buffer.pop_front();
        }
        state.sample_count += 1;

        // Evaluate only at packet boundaries (every 25 samples), matching Python rsudp.
        // Python rsudp evaluates once per ~250ms packet at 100 SPS.
        if !state.sample_count.is_multiple_of(25) || state.raw_buffer.len() < win_size {
            return None;
        }

        // Compute ratio: fresh filter + recursive STA/LTA over entire buffer
        let mut filters = butter_bandpass_sos(4, highpass, lowpass, 100.0);
        let csta = 1.0 / nsta as f64;
        let clta = 1.0 / nlta as f64;
        let mut sta = 0.0_f64;
        let mut lta = 1e-99_f64;
        let mut ratio_last = 0.0_f64;
        let mut ratio_max_tail = 0.0_f64;
        for (i, &raw) in state.raw_buffer.iter().enumerate() {
            let mut val = raw;
            for section in &mut filters {
                val = section.process(val);
            }
            let energy = val * val;
            sta = csta * energy + (1.0 - csta) * sta;
            lta = clta * energy + (1.0 - clta) * lta;
            ratio_last = sta / lta;
            // Match ObsPy: when ndat > nlta, first nlta values are zeroed.
            // Track max for the non-zeroed tail (positions >= nlta).
            if i >= nlta {
                ratio_max_tail = ratio_max_tail.max(ratio_last);
            }
        }

        // --- TRIGGER LOGIC ---
        // ALARM: use ratio_max_tail (matches Python rsudp stalta.max() after zeroing)
        // RESET: use ratio_last (matches Python rsudp stalta[-1])
        let threshold = self.config.threshold;
        let reset_threshold = self.config.reset_threshold;

        if !state.triggered {
            if ratio_max_tail > threshold {
                if !state.is_exceeding {
                    state.is_exceeding = true;
                    state.exceed_start = Some(timestamp);
                }
                if let Some(start) = state.exceed_start {
                    if (timestamp - start).num_milliseconds() as f64 / 1000.0 >= self.config.duration {
                        state.triggered = true;
                        state.max_ratio = ratio_max_tail;
                        state.is_exceeding = false;
                        return Some(AlertEvent {
                            timestamp, channel: id.to_string(), event_type: AlertEventType::Trigger,
                            ratio: ratio_max_tail, max_ratio: ratio_max_tail, message: "ALARM".to_string(),
                        });
                    }
                }
            } else {
                state.is_exceeding = false; state.exceed_start = None;
            }
        } else {
            state.max_ratio = state.max_ratio.max(ratio_max_tail);
            if ratio_last < reset_threshold {
                state.triggered = false;
                let mr = state.max_ratio;
                state.max_ratio = 0.0;

                return Some(AlertEvent {
                    timestamp, channel: id.to_string(), event_type: AlertEventType::Reset,
                    ratio: ratio_last, max_ratio: mr, message: "RESET".to_string(),
                });
            }
        }

        // --- PERIODIC STATUS REPORT ---
        if timestamp.timestamp_subsec_millis() < 10 {
             return Some(AlertEvent {
                timestamp, channel: id.to_string(), event_type: AlertEventType::Status,
                ratio: ratio_last, max_ratio: state.max_ratio.max(ratio_max_tail), message: "STATUS".to_string(),
            });
        }

        None
    }
}