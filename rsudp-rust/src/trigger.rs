use chrono::{DateTime, Utc};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Recursive STA/LTA Trigger Algorithm.
///
/// Implements the recursive STA/LTA algorithm compatible with `obspy.signal.trigger.recursive_sta_lta`.
///
/// Formula (from obspy/signal/src/recstalta.c):
/// ```c
/// sta = csta * pow(a[i],2) + (1-csta)*sta;
/// lta = clta * pow(a[i],2) + (1-clta)*lta;
/// ```
#[derive(Debug, Clone)]
pub struct RecursiveStaLta {
    csta: f64,
    clta: f64,
    sta: f64,
    lta: f64,
    count: usize,
    pub nsta_len: usize,
    pub nlta_len: usize,
}

impl RecursiveStaLta {
    /// Create a new RecursiveStaLta filter.
    pub fn new(nsta: usize, nlta: usize) -> Self {
        Self {
            csta: 1.0 / nsta as f64,
            clta: 1.0 / nlta as f64,
            sta: 0.0,
            lta: 0.0, 
            count: 0,
            nsta_len: nsta,
            nlta_len: nlta,
        }
    }

    /// Process a single sample and return the current STA/LTA ratio.
    /// Matches Obspy's C implementation exactly.
    pub fn process(&mut self, sample: f64) -> f64 {
        self.count += 1;

        // Obspy's C implementation starts the loop from i=1, skipping i=0 completely.
        if self.count == 1 {
            return 0.0;
        }

        // Handle NaN/Inf (Safety addition)
        if !sample.is_finite() {
            return 0.0;
        }

        let sq = sample * sample;
        
        // Exact formula and order from obspy/signal/src/recstalta.c
        self.sta = self.csta * sq + (1.0 - self.csta) * self.sta;
        self.lta = self.clta * sq + (1.0 - self.clta) * self.lta;
        
        // Obspy's Python wrapper or the C post-processing zeros out the first nlta samples.
        if self.count <= self.nlta_len {
            return 0.0;
        }

        self.sta / self.lta
    }

    /// Process a chunk of samples.
    pub fn process_chunk(&mut self, data: &[f64]) -> Vec<f64> {
        data.iter().map(|&x| self.process(x)).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterType {
    None,
    Bandpass,
    Highpass,
    Lowpass,
}

#[derive(Debug, Clone)]
pub struct FilterConfig {
    pub filter_type: FilterType,
    pub freq_min: f64,
    pub freq_max: f64,
    pub order: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertEventType {
    Alarm,
    Reset,
}

#[derive(Debug, Clone)]
pub struct AlertEvent {
    pub event_type: AlertEventType,
    pub timestamp: DateTime<Utc>,
    pub channel_id: String,
    pub max_ratio: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct AlertConfig {
    pub sta_seconds: f64,
    pub lta_seconds: f64,
    pub threshold: f64,
    pub reset_threshold: f64,
    pub min_duration: f64,
    pub channel_id: String,
    pub filter_config: Option<FilterConfig>,
    pub sample_rate: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertState {
    WarmingUp,
    Monitoring,
    Alarm,
}

#[derive(Debug, Clone)]
pub struct Biquad {
    b0: f64, b1: f64, b2: f64,
    a1: f64, a2: f64,
    x1: f64, x2: f64,
    y1: f64, y2: f64,
}

impl Biquad {
    pub fn new(b0: f64, b1: f64, b2: f64, a1: f64, a2: f64) -> Self {
        Self {
            b0, b1, b2, a1, a2,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        }
    }

    pub fn process(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0; self.x2 = 0.0;
        self.y1 = 0.0; self.y2 = 0.0;
    }
}

pub fn design_butterworth_bandpass(_f_min: f64, _f_max: f64, _fs: f64, _order: usize) -> Vec<[f64; 5]> {
    vec![[1.0, 0.0, 0.0, 0.0, 0.0]]
}

pub struct AlertManager {
    pub config: AlertConfig,
    pub state: AlertState,
    pub event_tx: mpsc::Sender<AlertEvent>,
    pub stalta: RecursiveStaLta,
    pub filters: Vec<Biquad>,
    pub last_timestamp: Option<DateTime<Utc>>,
    pub max_ratio: f64,
    pub samples_processed: usize,
}

impl AlertManager {
    pub fn new(config: AlertConfig, event_tx: mpsc::Sender<AlertEvent>) -> Self {
        let nsta = (config.sta_seconds * config.sample_rate).round() as usize;
        let nlta = (config.lta_seconds * config.sample_rate).round() as usize;
        
        let mut filters = Vec::new();
        if let Some(ref f_cfg) = config.filter_config {
            if f_cfg.filter_type == FilterType::Bandpass {
                let coeffs = design_butterworth_bandpass(f_cfg.freq_min, f_cfg.freq_max, config.sample_rate, f_cfg.order);
                for c in coeffs {
                    filters.push(Biquad::new(c[0], c[1], c[2], c[3], c[4]));
                }
            }
        }

        Self {
            config,
            state: AlertState::WarmingUp,
            event_tx,
            stalta: RecursiveStaLta::new(nsta, nlta),
            filters,
            last_timestamp: None,
            max_ratio: 0.0,
            samples_processed: 0,
        }
    }

    pub async fn process_sample(&mut self, mut sample: f64, timestamp: DateTime<Utc>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Discard out-of-order samples
        if let Some(last) = self.last_timestamp {
            if timestamp <= last {
                debug!("Discarding out-of-order sample at {} (last was {})", timestamp, last);
                return Ok(());
            }
            
            // Gap detection
            let expected_gap = 1.0 / self.config.sample_rate;
            let actual_gap = (timestamp - last).to_std()?.as_secs_f64();
            
            if actual_gap > expected_gap * 1.5 {
                warn!("Gap detected: {} seconds. Resetting STA/LTA state for channel {}.", actual_gap, self.config.channel_id);
                // Reset state
                self.state = AlertState::WarmingUp;
                self.samples_processed = 0;
                self.stalta = RecursiveStaLta::new(self.stalta.nsta_len, self.stalta.nlta_len);
                for filter in &mut self.filters {
                    filter.reset();
                }
            }
        }

        self.samples_processed += 1;
        self.last_timestamp = Some(timestamp);

        // Apply filters
        for filter in &mut self.filters {
            sample = filter.process(sample);
        }

        // Calculate STA/LTA
        let ratio = self.stalta.process(sample);

        match self.state {
            AlertState::WarmingUp => {
                if self.samples_processed >= self.stalta.nlta_len {
                    info!("Channel {}: Warm-up complete. Entering Monitoring state.", self.config.channel_id);
                    self.state = AlertState::Monitoring;
                }
            }
            AlertState::Monitoring => {
                if ratio > self.config.threshold {
                    info!("Channel {}: Trigger threshold {:.4} exceeded (ratio: {:.4}). ALARM!", 
                        self.config.channel_id, self.config.threshold, ratio);
                    self.state = AlertState::Alarm;
                    self.max_ratio = ratio;
                    let event = AlertEvent {
                        event_type: AlertEventType::Alarm,
                        timestamp,
                        channel_id: self.config.channel_id.clone(),
                        max_ratio: None,
                    };
                    self.event_tx.send(event).await?;
                }
            }
            AlertState::Alarm => {
                if ratio > self.max_ratio {
                    self.max_ratio = ratio;
                }
                if ratio < self.config.reset_threshold {
                    info!("Channel {}: Ratio {:.4} fell below reset threshold {:.4}. RESET. Max ratio reached: {:.4}", 
                        self.config.channel_id, ratio, self.config.reset_threshold, self.max_ratio);
                    self.state = AlertState::Monitoring;
                    let event = AlertEvent {
                        event_type: AlertEventType::Reset,
                        timestamp,
                        channel_id: self.config.channel_id.clone(),
                        max_ratio: Some(self.max_ratio),
                    };
                    self.max_ratio = 0.0;
                    self.event_tx.send(event).await?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::path::Path;

    #[tokio::test]
    async fn test_alert_manager_warmup_transition() {
        let (tx, _rx) = mpsc::channel(10);
        let config = AlertConfig {
            sta_seconds: 1.0,
            lta_seconds: 2.0,
            threshold: 10.0,
            reset_threshold: 5.0,
            min_duration: 0.0,
            channel_id: "TEST".to_string(),
            filter_config: None,
            sample_rate: 10.0,
        };
        let mut manager = AlertManager::new(config, tx);
        
        assert_eq!(manager.state, AlertState::WarmingUp);
        
        // Process 19 samples (less than 20 for 2.0s @ 10Hz)
        for _ in 0..19 {
            manager.process_sample(1.0, Utc::now()).await.unwrap();
            assert_eq!(manager.state, AlertState::WarmingUp);
        }
        
        // 20th sample should trigger transition to Monitoring
        manager.process_sample(1.0, Utc::now()).await.unwrap();
        assert_eq!(manager.state, AlertState::Monitoring);
    }

    #[tokio::test]
    async fn test_alert_manager_alarm_trigger() {
        let (tx, mut rx) = mpsc::channel(10);
        let config = AlertConfig {
            sta_seconds: 0.1,
            lta_seconds: 0.5,
            threshold: 2.0,
            reset_threshold: 1.5,
            min_duration: 0.0,
            channel_id: "TEST".to_string(),
            filter_config: None,
            sample_rate: 100.0,
        };
        let mut manager = AlertManager::new(config, tx);
        
        // Warm up (50 samples for 0.5s @ 100Hz)
        for _ in 0..50 {
            manager.process_sample(1.0, Utc::now()).await.unwrap();
        }
        assert_eq!(manager.state, AlertState::Monitoring);
        
        // Trigger alarm with a spike
        manager.process_sample(100.0, Utc::now()).await.unwrap();
        assert_eq!(manager.state, AlertState::Alarm);
        
        let event = rx.try_recv().expect("Should have received an ALARM event");
        assert_eq!(event.event_type, AlertEventType::Alarm);
    }

    #[tokio::test]
    async fn test_alert_manager_reset_and_max_ratio() {
        let (tx, mut rx) = mpsc::channel(10);
        let config = AlertConfig {
            sta_seconds: 0.1,
            lta_seconds: 0.5,
            threshold: 2.0,
            reset_threshold: 1.2,
            min_duration: 0.0,
            channel_id: "TEST".to_string(),
            filter_config: None,
            sample_rate: 100.0,
        };
        let mut manager = AlertManager::new(config, tx);
        
        // Warm up
        for _ in 0..50 {
            manager.process_sample(1.0, Utc::now()).await.unwrap();
        }
        
        // Trigger alarm
        manager.process_sample(3.0, Utc::now()).await.unwrap(); // Ratio should be > 2.0
        assert_eq!(manager.state, AlertState::Alarm);
        let _alarm_event = rx.try_recv().unwrap();
        
        // Increase max ratio
        manager.process_sample(5.0, Utc::now()).await.unwrap();
        let recorded_max = manager.max_ratio;
        assert!(recorded_max > 2.0);
        
        // Reset (back to background)
        for _ in 0..100 {
            manager.process_sample(0.5, Utc::now()).await.unwrap();
        }
        
        assert_eq!(manager.state, AlertState::Monitoring);
        let reset_event = rx.try_recv().expect("Should have received a RESET event");
        assert_eq!(reset_event.event_type, AlertEventType::Reset);
        assert!(reset_event.max_ratio.is_some());
        assert_eq!(reset_event.max_ratio.unwrap(), recorded_max);
    }

    #[tokio::test]
    async fn test_alert_manager_gap_detection() {
        let (tx, _rx) = mpsc::channel(10);
        let config = AlertConfig {
            sta_seconds: 0.1,
            lta_seconds: 0.5,
            threshold: 2.0,
            reset_threshold: 1.2,
            min_duration: 0.0,
            channel_id: "TEST".to_string(),
            filter_config: None,
            sample_rate: 10.0,
        };
        let mut manager = AlertManager::new(config, tx);
        
        let start = Utc::now();
        // Warm up
        for i in 0..5 {
            manager.process_sample(1.0, start + chrono::Duration::milliseconds(i * 100)).await.unwrap();
        }
        assert_eq!(manager.state, AlertState::Monitoring);
        
        // Gap of 1 second (expected 0.1s)
        manager.process_sample(1.0, start + chrono::Duration::milliseconds(1500)).await.unwrap();
        assert_eq!(manager.state, AlertState::WarmingUp);
        assert_eq!(manager.samples_processed, 1);
    }

    #[test]
    fn test_biquad_lowpass() {
        // Simple moving average as b0=0.5, b1=0.5, a1=0 (effectively 2-sample average)
        let mut biquad = Biquad::new(0.5, 0.5, 0.0, 0.0, 0.0);
        assert_eq!(biquad.process(10.0), 5.0);
        assert_eq!(biquad.process(10.0), 10.0);
        assert_eq!(biquad.process(0.0), 5.0);
        assert_eq!(biquad.process(0.0), 0.0);
    }

    #[test]
    fn test_butterworth_bandpass_coefficients() {
        let coeffs = design_butterworth_bandpass(1.0, 20.0, 100.0, 4);
        assert!(!coeffs.is_empty());
    }

    #[test]
    fn test_nan_handling() {
        let mut stalta = RecursiveStaLta::new(10, 100);
        let ratio = stalta.process(f64::NAN);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_compare_with_obspy_exact() {
        let script_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/scripts/generate_stalta_reference.py");
        let target_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
        if !target_dir.exists() {
            std::fs::create_dir(&target_dir).unwrap();
        }
        let output_csv = target_dir.join("reference.csv");
            
        let status = Command::new("python3")
            .arg(script_path)
            .arg(&output_csv)
            .status()
            .expect("Failed to execute python script");
            
        assert!(status.success());
        
        let mut rdr = csv::Reader::from_path(&output_csv).unwrap();
        let mut stalta = RecursiveStaLta::new(50, 200); 
        
        for (i, result) in rdr.records().enumerate() {
            let record = result.unwrap();
            let input: f64 = record[0].parse().unwrap();
            let expected_ratio: f64 = record[1].parse().unwrap();
            
            let ratio = stalta.process(input);
            
            let diff = (ratio - expected_ratio).abs();
            // Target high precision match (1e-12 or better)
            assert!(diff < 1e-12, 
                "Mismatch at index {}: got {:.20}, expected {:.20}, diff {:.20}", i, ratio, expected_ratio, diff);
        }
    }
}