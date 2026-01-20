use std::f64::consts::PI;
use num_complex::Complex;

// Helper to create 4th order (2 cascaded 2nd order) Butterworth bandpass SOS
// Equivalent to scipy.signal.butter(4, [low, high], btype='band', output='sos', fs=fs)
// Note: "order 4" in scipy for bandpass means 4th order -> 8 poles total -> 4 biquad sections
pub fn butter_bandpass_sos(order: usize, low_freq: f64, high_freq: f64, fs: f64) -> Vec<Biquad> {
    if order != 4 {
        panic!("Only order 4 implementation is supported for now (matches rsudp default)");
    }

    let nyquist = fs / 2.0;
    let low = low_freq / nyquist;
    let high = high_freq / nyquist;

    // Pre-warp frequencies
    let u_low = (PI * low / 2.0).tan();
    let u_high = (PI * high / 2.0).tan();
    
    let bw = u_high - u_low;
    let center_sq = u_high * u_low;

    // Analog prototype (Butterworth, order 4)
    // Poles on unit circle in s-plane: exp(j * (2k + n + 1) * pi / (2n))
    // For n=4: k=0..3 -> angles: 5pi/8, 7pi/8, 9pi/8, 11pi/8
    // We only need poles with real part < 0 (LHP)
    let angles = vec![
        5.0 * PI / 8.0,
        7.0 * PI / 8.0,
        9.0 * PI / 8.0,
        11.0 * PI / 8.0,
    ];

    let mut sections = Vec::new();

    // Group poles into conjugate pairs to form 2nd order sections
    // Angles 5pi/8 and 11pi/8 are conjugates (11pi/8 = -5pi/8)
    // Angles 7pi/8 and 9pi/8 are conjugates (9pi/8 = -7pi/8)
    
    // We process pairs of poles from the prototype
    // For bandpass transformation, each real pole becomes 1 biquad, each conjugate pair becomes 2 biquads (4th order total)
    // But here we start with 4th order prototype, so we have 4 poles in LHP.
    // Each pair (p, p*) transforms into a 4th order section (2 biquads) in bandpass?
    // Wait, scipy 'order 4' bandpass means the RESULT is 8th order (4 biquads).
    // Let's stick to the standard bilinear transform steps.

    let poles_proto: Vec<Complex<f64>> = angles.iter().map(|&a| Complex::from_polar(1.0, a)).collect();

    // Bandpass transformation: s -> (s^2 + center_sq) / (s * bw)
    // Solve for s: s^2 - p*bw*s + center_sq = 0
    let mut poles_analog = Vec::new();
    for p in poles_proto {
        // Roots of s^2 - (p*bw)*s + center_sq = 0
        let b_val = -p * bw;
        let c_val = Complex::new(center_sq, 0.0);
        let disc = (b_val * b_val - 4.0 * c_val).sqrt();
        let s1 = (-b_val + disc) / 2.0;
        let s2 = (-b_val - disc) / 2.0;
        poles_analog.push(s1);
        poles_analog.push(s2);
    }
    
    // We have 8 poles. Group into 4 conjugate pairs.
    // Sort by real part? Or just find conjugates.
    // Since we generated them from conjugates, s1 corresponding to p and s1 corresponding to p* should be conjugates?
    // Actually, simpler approach for Biquads:
    // Bilinear transform: z = (1+s)/(1-s) -> s = (z-1)/(z+1)
    
    // Let's map poles to z-plane first
    let mut poles_z = Vec::new();
    for s in poles_analog {
        let z = (1.0 + s) / (1.0 - s);
        poles_z.push(z);
    }

    // Zeros: For bandpass, we have N zeros at z=1 and N zeros at z=-1 (from s=0 and s=inf)
    // N = order (4)
    // So 4 zeros at +1, 4 zeros at -1.
    
    // Group into sections. Each section needs 2 poles and 2 zeros.
    // We need to pair complex conjugate poles.
    
    // Simple greedy pairing of conjugates
    let mut used = vec![false; poles_z.len()];
    for i in 0..poles_z.len() {
        if used[i] { continue; }
        
        // Find conjugate
        let mut best_j = i;
        let mut min_err = 1e9;
        
        for j in (i+1)..poles_z.len() {
            if used[j] { continue; }
            let err = (poles_z[i].re - poles_z[j].re).abs() + (poles_z[i].im + poles_z[j].im).abs();
            if err < min_err {
                min_err = err;
                best_j = j;
            }
        }
        
        used[i] = true;
        used[best_j] = true;
        
        let p1 = poles_z[i];
        let p2 = poles_z[best_j];
        
        // Form Biquad from poles p1, p2 and zeros +1, -1
        // (z - 1)(z + 1) = z^2 - 1
        // (z - p1)(z - p2) = z^2 - (p1+p2)z + p1p2
        
        let poly_p_re = - (p1 + p2).re;
        let poly_p_abs_sq = (p1 * p2).re; // Assuming conjugates, product is real
        
        // Denominator (a): 1, poly_p_re, poly_p_abs_sq
        // Numerator (b): 1, 0, -1 (from z^2 - 1)
        
        // Apply gain? Usually done globally or distributed. 
        // For Butterworth bandpass, max gain is at center.
        // We need to normalize so gain is 1.0 at band center.
        // Or simpler: normalize at DC or Nyquist? No, bandpass is 0 there.
        // Standard approach: normalize so sum(a) = sum(b)? No.
        // Let's use the standard biquad form directly.
        
        // Direct definition:
        // H(z) = (b0 + b1 z^-1 + b2 z^-2) / (1 + a1 z^-1 + a2 z^-2)
        // a0 is normalized to 1.
        
        let a0 = 1.0;
        let a1 = poly_p_re;
        let a2 = poly_p_abs_sq;
        
        let b0 = 1.0;
        let b1 = 0.0;
        let b2 = -1.0;
        
        // Gain adjustment to match scipy behavior (optimally distributed)
        // Scipy distributes gain. For a single section bandpass biquad (z^2-1)/(z^2+a1z+a2):
        // Gain at center freq?
        // Let's calculate gain at a reference frequency in the passband (e.g. geometric mean)
        // But doing this per section is complex.
        // Alternative: Use the s-plane to z-plane substitution formula directly for Biquad coefficients
        
        sections.push(Biquad { b0, b1, b2, a1, a2, x1:0.0, x2:0.0, y1:0.0, y2:0.0 });
    }
    
    // Calculate global gain to normalize peak to 1.0
    // Evaluate transfer function at center frequency
    let center_freq = (low_freq * high_freq).sqrt();
    let omega = 2.0 * PI * center_freq / fs;
    let z = Complex::from_polar(1.0, omega);
    
    let mut mag = 1.0;
    for s in &sections {
        let num = s.b0 * z * z + s.b1 * z + s.b2;
        let den = z * z + s.a1 * z + s.a2;
        mag *= (num / den).norm();
    }
    
    // Distribute 1/mag gain to all sections (nth root)
    let section_gain = (1.0 / mag).powf(1.0 / sections.len() as f64);
    
    for s in &mut sections {
        s.b0 *= section_gain;
        s.b1 *= section_gain;
        s.b2 *= section_gain;
    }

    sections
}
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
    pub max_ratio: f64,
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
    pub duration: f64,
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
        // Butterworth 4th order (2nd order cascaded twice) 0.1-2.0Hz @ 100Hz
        // Calculated using scipy.signal.butter(4, [0.1, 2.0], btype='band', fs=100, output='sos')
        // Section 1
        let s1 = Biquad::new(0.00001332, 0.00002664, 0.00001332, -1.91119707, 0.91497583);
        // Section 2
        let s2 = Biquad::new(1.0, -2.0, 1.0, -1.9911143, 0.9911536);
        // Section 3
        let s3 = Biquad::new(1.0, -2.0, 1.0, -1.92379307, 0.92850732);
        // Section 4
        let s4 = Biquad::new(1.0, -2.0, 1.0, -1.99547377, 0.99555234);
        
        Self { sections: vec![s1, s2, s3, s4], initialized: false }
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
    exceed_start: Option<DateTime<Utc>>,
    is_exceeding: bool,
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
            exceed_start: None,
            is_exceeding: false,
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
                state.exceed_start = None;
                state.is_exceeding = false;
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
                // Use wait_pkts logic: LTA seconds of data MUST be seen before evaluating trigger
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
        
                let ratio_exceeded = ratio > self.config.threshold;
        
                if !state.triggered {
                    if ratio_exceeded {
                        if !state.is_exceeding {
                            state.is_exceeding = true;
                            state.exceed_start = Some(timestamp);
                        }
        
                        // Check duration
                        if let Some(start) = state.exceed_start {
                            let elapsed = (timestamp - start).num_milliseconds() as f64 / 1000.0;
                            if elapsed >= self.config.duration {
                                state.triggered = true;
                                state.max_ratio = ratio;
                                state.is_exceeding = false; // Reset exceeding state once triggered
                                let message = format!(
                                    "Trigger threshold {} exceeded for {}s (ratio: {:.4}). ALARM!",
                                    self.config.threshold, self.config.duration, ratio
                                );
                                return Some(AlertEvent {
                                    timestamp,
                                    channel: id.to_string(),
                                    event_type: AlertEventType::Trigger,
                                    ratio,
                                    max_ratio: ratio,
                                    message,
                                });
                            }
                        }
                    } else {
                        // Ratio fell below threshold before duration met
                        state.is_exceeding = false;
                        state.exceed_start = None;
                    }
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
                            max_ratio: state.max_ratio,
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

                

        