use crate::filter::{BiquadChain, deconvolve_response};
use crate::intensity::IntensityResult;
use crate::parser::stationxml::ChannelResponse;
use crate::trigger::AlertEvent;
use crate::web::history::{AlertHistoryManager, SharedHistory};
use crate::web::spectrogram::compute_spectrogram;
use axum::{
    extract::ws::Message,
    extract::{State, WebSocketUpgrade},
    response::Response,
};
use chrono::{DateTime, Utc, Duration};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::broadcast;
use tracing::{warn, debug};
use rustfft::{FftPlanner, num_complex::Complex};

use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSettings {
    pub scale: f64,
    pub window_seconds: f64,
    pub save_pct: f64,
    pub output_dir: PathBuf,
    pub deconvolve: bool,
    pub units: String,
    pub eq_screenshots: bool,
    pub show_spectrogram: bool,
    pub spectrogram_freq_min: f64,
    pub spectrogram_freq_max: f64,
    pub spectrogram_log_y: bool,
    pub filter_waveform: bool,
    pub filter_highpass: f64,
    pub filter_lowpass: f64,
    pub filter_corners: usize,
}

#[derive(Debug, Clone)]
pub struct ChannelBuffer {
    pub data: VecDeque<f64>,
    pub end_time: DateTime<Utc>, // Timestamp of the last sample in buffer
    pub sample_rate: f64,
}

impl ChannelBuffer {
    pub fn new(capacity: usize, sample_rate: f64) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            end_time: Utc::now(), // Will be updated on first push
            sample_rate,
        }
    }

    pub fn push_segment(&mut self, start_time: DateTime<Utc>, samples: &[f64], max_len: usize) {
        // MiniSEED records might overlap or have gaps, but assuming continuous for now.
        // We trust the record's start time.
        // The end time of this segment is start_time + (samples.len() - 1) * dt
        
        // Update end_time to the time of the LAST sample in this new batch
        // duration of N samples is (N-1)*dt
        let segment_duration = Duration::milliseconds((((samples.len() - 1) as f64 / self.sample_rate) * 1000.0) as i64);
        self.end_time = start_time + segment_duration;

        for &s in samples {
            if self.data.len() >= max_len {
                self.data.pop_front();
            }
            self.data.push_back(s);
        }
    }

    /// Extract samples for a specific time window [start, end]
    pub fn extract_window(&self, start: DateTime<Utc>, duration_sec: f64) -> (Vec<f64>, DateTime<Utc>) {
        let dt = 1.0 / self.sample_rate;
        let samples_needed = (duration_sec * self.sample_rate) as usize;
        
        // Calculate the time of the FIRST sample in the buffer
        let buffer_duration_ms = ((self.data.len() as f64 - 1.0) * dt * 1000.0) as i64;
        let buffer_start_time = self.end_time - Duration::milliseconds(buffer_duration_ms);

        debug!("Buffer Status: Len={}, End={}, Start={}", self.data.len(), self.end_time, buffer_start_time);
        debug!("Request: Start={}, Duration={}s", start, duration_sec);

        // Find index corresponding to requested start time
        let offset_ms = (start - buffer_start_time).num_milliseconds();
        let offset_samples = (offset_ms as f64 / 1000.0 * self.sample_rate).round() as isize;
        
        debug!("Calculated Offset: {}ms ({} samples)", offset_ms, offset_samples);

        if offset_samples < 0 {
            warn!("Requested time {} is before buffer start {}", start, buffer_start_time);
             let end_idx = samples_needed.min(self.data.len());
             return (self.data.range(0..end_idx).cloned().collect(), buffer_start_time);
        }

        let start_idx = offset_samples as usize;
        if start_idx >= self.data.len() {
            // Requested future data
            return (Vec::new(), start);
        }

        let end_idx = (start_idx + samples_needed).min(self.data.len());
        let slice = self.data.range(start_idx..end_idx).cloned().collect();
        
        // The actual start time of the slice
        let actual_start_time = buffer_start_time + Duration::milliseconds(((start_idx as f64 * dt) * 1000.0) as i64);
        
        (slice, actual_start_time)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    Waveform {
        channel: String,
        timestamp: DateTime<Utc>,
        samples: Vec<f64>,
    },
    Alert(AlertEvent),
    Intensity(IntensityResult),
    AlertStart {
        id: uuid::Uuid,
        channel: String,
        timestamp: DateTime<Utc>,
    },
    AlertEnd {
        id: uuid::Uuid,
        channel: String,
        timestamp: DateTime<Utc>,
        max_ratio: f64,
        message: String,
    },
    #[serde(skip)]
    Spectrogram {
        channel: String,
        timestamp: DateTime<Utc>,
        sample_rate: f64,
    },
}

#[derive(Clone)]
pub struct WebState {
    pub tx: broadcast::Sender<WsMessage>,
    pub settings: Arc<RwLock<PlotSettings>>,
    pub history: SharedHistory,
    pub waveform_buffers: Arc<Mutex<HashMap<String, ChannelBuffer>>>, // Updated type
    pub alert_max_intensities: Arc<Mutex<HashMap<uuid::Uuid, f64>>>,
    pub station_name: Arc<RwLock<String>>,
    pub sensitivity_map: Arc<RwLock<HashMap<String, f64>>>,
    pub response_map: Arc<RwLock<HashMap<String, ChannelResponse>>>,
}

impl Default for WebState {
    fn default() -> Self {
        Self::new()
    }
}

impl WebState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            tx,
            settings: Arc::new(RwLock::new(PlotSettings {
                scale: 1.0,
                window_seconds: 90.0,
                save_pct: 0.7,
                output_dir: PathBuf::from("."),
                deconvolve: false,
                units: "counts".to_string(),
                eq_screenshots: false,
                show_spectrogram: true,
                spectrogram_freq_min: 0.0,
                spectrogram_freq_max: 50.0,
                spectrogram_log_y: false,
                filter_waveform: false,
                filter_highpass: 0.7,
                filter_lowpass: 2.0,
                filter_corners: 4,
            })),
            history: Arc::new(Mutex::new(AlertHistoryManager::new())),
            waveform_buffers: Arc::new(Mutex::new(HashMap::new())),
            alert_max_intensities: Arc::new(Mutex::new(HashMap::new())),
            station_name: Arc::new(RwLock::new(String::new())),
            sensitivity_map: Arc::new(RwLock::new(HashMap::new())),
            response_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WsMessage> {
        self.tx.subscribe()
    }

    pub async fn broadcast_alert(&self, ev: AlertEvent) {
        let _ = self.tx.send(WsMessage::Alert(ev));
    }

    pub async fn broadcast_waveform(
        &self,
        channel: String,
        timestamp: DateTime<Utc>,
        samples: Vec<f64>,
    ) {
        let _ = self.tx.send(WsMessage::Waveform {
            channel,
            timestamp,
            samples,
        });
    }

    pub async fn broadcast_intensity(&self, res: IntensityResult) {
        let _ = self.tx.send(WsMessage::Intensity(res));
    }

    pub async fn broadcast_alert_start(&self, id: uuid::Uuid, channel: String, timestamp: DateTime<Utc>) {
        let _ = self.tx.send(WsMessage::AlertStart { id, channel, timestamp });
    }

    pub async fn broadcast_alert_end(&self, id: uuid::Uuid, channel: String, timestamp: DateTime<Utc>, max_ratio: f64, message: String) {
        let _ = self.tx.send(WsMessage::AlertEnd { id, channel, timestamp, max_ratio, message });
    }
}

/// Serialize a spectrogram packet as binary (type 0x04) with f32 compressed PSD values.
/// Format: [0x04][channelIdLen:u8][channelId:utf8][timestamp:i64le(μs)][sampleRate:f32le][hopDuration:f32le][frequencyBins:u16le][columnsCount:u16le][data:f32le[columnsCount × frequencyBins]]
/// The frontend performs per-frame min-max normalization (matching rsudp/matplotlib's imshow auto-scaling).
pub fn serialize_spectrogram_f32_packet(
    channel_id: &str,
    timestamp_us: i64,
    sample_rate: f32,
    hop_duration: f32,
    frequency_bins: u16,
    columns: &[Vec<f32>],
) -> Vec<u8> {
    let channel_bytes = channel_id.as_bytes();
    let cols_count = columns.len() as u16;
    let data_size = (cols_count as usize) * (frequency_bins as usize) * 4; // f32 = 4 bytes

    let total_size = 1 + 1 + channel_bytes.len() + 8 + 4 + 4 + 2 + 2 + data_size;
    let mut buf = Vec::with_capacity(total_size);

    buf.push(0x04); // type: f32 spectrogram
    buf.push(channel_bytes.len() as u8);
    buf.extend_from_slice(channel_bytes);
    buf.extend_from_slice(&timestamp_us.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&hop_duration.to_le_bytes());
    buf.extend_from_slice(&frequency_bins.to_le_bytes());
    buf.extend_from_slice(&cols_count.to_le_bytes());

    // Data: column-major layout, f32le per value
    for col in columns {
        for &val in col {
            buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    buf
}

/// Serialize a waveform packet as binary (type 0x00)
/// Format: [0x00][channelIdLen:u8][channelId:utf8][timestamp:i64le(μs)][sampleRate:f32le][samplesCount:u32le][samples:f32le[]]
pub fn serialize_waveform_packet(
    channel_id: &str,
    timestamp_us: i64,
    sample_rate: f32,
    samples: &[f64],
) -> Vec<u8> {
    let channel_bytes = channel_id.as_bytes();
    let samples_count = samples.len() as u32;

    let total_size = 1 + 1 + channel_bytes.len() + 8 + 4 + 4 + (samples.len() * 4);
    let mut buf = Vec::with_capacity(total_size);

    buf.push(0x00); // type
    buf.push(channel_bytes.len() as u8); // channelIdLen
    buf.extend_from_slice(channel_bytes); // channelId
    buf.extend_from_slice(&timestamp_us.to_le_bytes()); // timestamp i64le
    buf.extend_from_slice(&sample_rate.to_le_bytes()); // sampleRate f32le
    buf.extend_from_slice(&samples_count.to_le_bytes()); // samplesCount u32le

    for &s in samples {
        buf.extend_from_slice(&(s as f32).to_le_bytes());
    }

    buf
}

/// FFT parameters (matches static plot: NFFT=128, 90% overlap, hop=13)
const NFFT: usize = 128;
const HOP: usize = 13;

/// Hanning window coefficients
fn hanning_window(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| 0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / (n - 1) as f64).cos()))
        .collect()
}

/// Per-channel state for incremental FFT computation.
/// Returns raw compressed PSD values (f32) — normalization is done on the frontend
/// per-frame, matching rsudp/matplotlib's imshow() auto-scaling.
struct FftChannelState {
    carry_buf: Vec<f64>,
}

impl FftChannelState {
    fn new() -> Self {
        Self {
            carry_buf: Vec::new(),
        }
    }
}

/// Compute new spectrogram columns incrementally from carry buffer + new samples.
/// Returns raw compressed PSD values (f32) — no normalization.
/// The frontend performs per-frame min-max normalization matching rsudp/matplotlib's imshow() auto-scaling.
fn compute_incremental_columns(
    state: &mut FftChannelState,
    new_samples: &[f64],
    hann: &[f64],
    fft: &std::sync::Arc<dyn rustfft::Fft<f64>>,
    sample_rate: f64,
) -> Vec<Vec<f32>> {
    let freq_bins = NFFT / 2 + 1;

    // PSD normalization factor: Fs × Σ(window²)
    // Matches matplotlib's _spectral_helper default for mode='psd'
    let window_power_sum: f64 = hann.iter().map(|w| w * w).sum();
    let psd_norm = sample_rate * window_power_sum;

    // Combine carry buffer with new samples
    let mut combined = Vec::with_capacity(state.carry_buf.len() + new_samples.len());
    combined.extend_from_slice(&state.carry_buf);
    combined.extend_from_slice(new_samples);

    if combined.len() < NFFT {
        state.carry_buf = combined;
        return Vec::new();
    }

    let mut columns: Vec<Vec<f32>> = Vec::new();
    let mut pos = 0;

    while pos + NFFT <= combined.len() {
        let chunk = &combined[pos..pos + NFFT];
        let mean = chunk.iter().sum::<f64>() / NFFT as f64;

        let mut buffer: Vec<Complex<f64>> = chunk.iter().zip(hann.iter())
            .map(|(&s, &w)| Complex { re: (s - mean) * w, im: 0.0 })
            .collect();

        fft.process(&mut buffer);

        // PSD normalization + one-sided correction + power-law compression on linear PSD
        // Matches rsudp's Pxx ** (1/10) on matplotlib's linear PSD output
        let compressed: Vec<f32> = buffer.iter().take(freq_bins).enumerate()
            .map(|(k, c)| {
                let mag_sq = c.re * c.re + c.im * c.im;
                let mut psd = mag_sq / psd_norm;
                // One-sided spectrum: double non-DC, non-Nyquist bins
                if k > 0 && k < freq_bins - 1 { psd *= 2.0; }
                // Power-law compression on linear PSD (matching rsudp's sg ** (1/10))
                psd.powf(0.1) as f32
            })
            .collect();

        columns.push(compressed);
        pos += HOP;
    }

    // Save unprocessed samples as carry for next batch
    state.carry_buf = combined[pos..].to_vec();

    columns
}

#[derive(Debug, Deserialize)]
struct BackfillRequest {
    #[serde(rename = "type")]
    msg_type: String,
    last_timestamp: Option<String>,
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<WebState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: axum::extract::ws::WebSocket, state: WebState) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast before backfill so we don't miss messages
    let mut rx = state.subscribe();

    // Wait for BackfillRequest from client (with timeout)
    let backfill_last_ts: Option<DateTime<Utc>> = {
        let timeout = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            async {
                while let Some(Ok(msg)) = receiver.next().await {
                    if let Message::Text(text) = msg {
                        if let Ok(req) = serde_json::from_str::<BackfillRequest>(&text) {
                            if req.msg_type == "BackfillRequest" {
                                debug!("Received BackfillRequest: {:?}", req.last_timestamp);
                                return req.last_timestamp.and_then(|ts| {
                                    DateTime::parse_from_rfc3339(&ts).ok().map(|dt| dt.with_timezone(&Utc))
                                });
                            }
                        }
                    }
                    break;
                }
                None
            }
        ).await;
        timeout.unwrap_or(None)
    };

    // Read deconvolution settings once for this connection
    let (deconvolve, units, window_seconds) = {
        let settings = state.settings.read().unwrap();
        (settings.deconvolve, settings.units.clone(), settings.window_seconds)
    };
    let resp_map = {
        let rm = state.response_map.read().unwrap();
        rm.clone()
    };
    let sens_map = {
        let sm = state.sensitivity_map.read().unwrap();
        sm.clone()
    };

    // Pre-filter for deconvolution: matches rsudp's [0.1, 0.6, 0.95*sps, sps].
    // At 100 Hz sps: [0.1, 0.6, 95.0, 100.0]. Since our FFT only covers
    // frequencies up to Nyquist (sps/2 = 50 Hz), the upper taper at 95-100 Hz
    // is entirely above Nyquist, meaning ALL frequency bins pass with weight 1.0.
    // This matches rsudp/obspy behavior — no high-frequency attenuation.
    let deconv_water_level = 4.5; // dB, matches rsudp default

    // Helper: apply batch deconvolution (for backfill)
    let deconvolve_batch = |samples: &[f64], channel_id: &str, sample_rate: f64| -> Vec<f64> {
        if !deconvolve {
            return samples.to_vec();
        }
        // Try frequency-domain deconvolution with poles/zeros
        if let Some(response) = resp_map.get(channel_id) {
            if !response.poles.is_empty() {
                let pre_filt = [0.1, 0.6, 0.95 * sample_rate, sample_rate];
                let mut deconv = deconvolve_response(samples, response, sample_rate, pre_filt, deconv_water_level);
                // Match rsudp: demean the deconvolved signal before filtering
                // (rsudp calls stream.detrend('demean') after remove_response)
                let vel_mean = deconv.iter().sum::<f64>() / deconv.len().max(1) as f64;
                for s in &mut deconv {
                    *s -= vel_mean;
                }
                // Apply extra divisor for GRAV units
                if units.to_uppercase() == "GRAV" {
                    for s in &mut deconv {
                        *s /= 9.81;
                    }
                }
                return deconv;
            }
        }
        // Fallback: simple scalar division
        let sensitivity = match sens_map.get(channel_id) {
            Some(&s) if s > 0.0 => s,
            _ => return samples.to_vec(),
        };
        let extra_divisor = if units.to_uppercase() == "GRAV" { 9.81 } else { 1.0 };
        samples.iter().map(|&s| s / sensitivity / extra_divisor).collect()
    };

    // Per-channel raw sample context buffers for live FFT deconvolution.
    // We keep the last `deconv_context` samples for each channel so that each incoming
    // packet can be deconvolved with sufficient frequency resolution.
    // Use the display duration (window_seconds) as context size, matching rsudp's approach
    // of deconvolving the entire accumulated stream before slicing to the display window.
    // sample_rate is not yet known here; we'll compute actual capacity lazily per-channel.
    let deconv_context_seconds = window_seconds;
    let mut raw_context_bufs: HashMap<String, VecDeque<f64>> = HashMap::new();

    // Read filter and spectrogram settings (reactive — re-read periodically in loop)
    let (mut filter_enabled, mut filter_highpass, mut filter_lowpass, mut filter_corners) = {
        let s = state.settings.read().unwrap();
        (s.filter_waveform, s.filter_highpass, s.filter_lowpass, s.filter_corners)
    };
    let (mut spec_freq_min, mut spec_freq_max) = {
        let s = state.settings.read().unwrap();
        (s.spectrogram_freq_min, s.spectrogram_freq_max)
    };

    // Note: bandpass filter is applied fresh (forward-only) to the full deconvolved
    // context buffer each packet, matching rsudp's obspy default (zerophase=False).

    // Send backfill data
    let _window_seconds = {
        let settings = state.settings.read().unwrap();
        settings.window_seconds
    };

    // Extract backfill data from buffers (lock scope limited, no await inside)
    struct BackfillData {
        channel_id: String,
        samples: Vec<f64>,
        sample_rate: f64,
        timestamp_us: i64,
    }

    let backfill_items: Vec<BackfillData> = {
        let buffers = state.waveform_buffers.lock().unwrap();
        let mut items = Vec::new();

        for (channel_id, buffer) in buffers.iter() {
            if buffer.data.is_empty() {
                continue;
            }

            let samples: Vec<f64>;
            let start_time: DateTime<Utc>;

            if let Some(last_ts) = backfill_last_ts {
                let dt = 1.0 / buffer.sample_rate;
                let buffer_duration_ms = ((buffer.data.len() as f64 - 1.0) * dt * 1000.0) as i64;
                let buffer_start = buffer.end_time - Duration::milliseconds(buffer_duration_ms);

                if last_ts < buffer_start {
                    samples = buffer.data.iter().cloned().collect();
                    start_time = buffer_start;
                } else if last_ts >= buffer.end_time {
                    continue;
                } else {
                    let offset_ms = (last_ts - buffer_start).num_milliseconds();
                    let offset_samples = (offset_ms as f64 / 1000.0 * buffer.sample_rate).round() as usize;
                    let start_idx = offset_samples.min(buffer.data.len());
                    samples = buffer.data.range(start_idx..).cloned().collect();
                    start_time = buffer_start + Duration::milliseconds(((start_idx as f64 / buffer.sample_rate) * 1000.0) as i64);
                }
            } else {
                let dt = 1.0 / buffer.sample_rate;
                let buffer_duration_ms = ((buffer.data.len() as f64 - 1.0) * dt * 1000.0) as i64;
                start_time = buffer.end_time - Duration::milliseconds(buffer_duration_ms);
                samples = buffer.data.iter().cloned().collect();
            }

            if samples.is_empty() {
                continue;
            }

            items.push(BackfillData {
                channel_id: channel_id.clone(),
                samples,
                sample_rate: buffer.sample_rate,
                timestamp_us: start_time.timestamp_micros(),
            });
        }
        items
    }; // MutexGuard dropped here

    // Prepare FFT state early so backfill can initialize running_max
    let hann = hanning_window(NFFT);
    let fft = {
        let mut planner = FftPlanner::new();
        planner.plan_fft_forward(NFFT)
    };
    let mut fft_states: HashMap<String, FftChannelState> = HashMap::new();

    // Now send backfill data (safe to await)
    let mut backfill_channels = Vec::new();
    for item in &backfill_items {
        let deconv_samples = deconvolve_batch(&item.samples, &item.channel_id, item.sample_rate);

        // Apply forward-only bandpass filter to backfill data, matching rsudp's actual behavior
        // (obspy default zerophase=False uses sosfilt, not sosfiltfilt).
        // The startup transient is trimmed from the beginning.
        let (waveform_samples, waveform_ts) = if filter_enabled && filter_highpass > 0.0 && filter_lowpass > filter_highpass {
            let mut chain = BiquadChain::bandpass(filter_corners, filter_highpass, filter_lowpass, item.sample_rate);
            let all_filtered = chain.process_vec(&deconv_samples);

            // Trim startup transient: ~10 cycles of the lowest frequency
            let trim_seconds = (10.0 / filter_highpass).min(30.0);
            let trim_n = (trim_seconds * item.sample_rate) as usize;

            let (filtered, adj_ts) = if all_filtered.len() > trim_n + (item.sample_rate as usize) {
                (all_filtered[trim_n..].to_vec(),
                 item.timestamp_us + (trim_n as f64 / item.sample_rate * 1_000_000.0) as i64)
            } else {
                // Small buffer: fallback to demeaned unfiltered
                let mean = deconv_samples.iter().sum::<f64>() / deconv_samples.len().max(1) as f64;
                (deconv_samples.iter().map(|&x| x - mean).collect(), item.timestamp_us)
            };
            (filtered, adj_ts)
        } else {
            (deconv_samples.clone(), item.timestamp_us)
        };

        let waveform_packet = serialize_waveform_packet(
            &item.channel_id,
            waveform_ts,
            item.sample_rate as f32,
            &waveform_samples,
        );
        if sender.send(Message::Binary(waveform_packet)).await.is_err() {
            return;
        }

        // Spectrogram always uses deconvolved (unfiltered) data, matching rsudp default
        // (rsudp's filter_spectrogram=False by default — spectrogram is independent of waveform filter)
        // This avoids bandpass filter startup transient noise contaminating spectrogram normalization
        let spec_input = &deconv_samples;

        if spec_input.len() >= NFFT {
            let hop_duration = HOP as f32 / item.sample_rate as f32;
            let noverlap = NFFT - HOP;
            let freq_bins = NFFT / 2 + 1;

            // Compute raw spectrogram
            let raw_spec = compute_spectrogram(spec_input, item.sample_rate, NFFT, noverlap);

            if !raw_spec.data.is_empty() {
                // Compress all backfill columns: PSD^0.1 → f32
                // No normalization — frontend does per-frame min-max (matching rsudp/matplotlib)
                let columns: Vec<Vec<f32>> = raw_spec.data.iter().map(|row| {
                    row.iter().map(|&psd| psd.max(0.0).powf(0.1) as f32).collect()
                }).collect();

                let spec_packet = serialize_spectrogram_f32_packet(
                    &item.channel_id,
                    item.timestamp_us,
                    item.sample_rate as f32,
                    hop_duration,
                    freq_bins as u16,
                    &columns,
                );
                if sender.send(Message::Binary(spec_packet)).await.is_err() {
                    return;
                }

                // Initialize FFT state with carry buffer
                let num_cols = raw_spec.data.len();
                let carry_start = num_cols * HOP;

                fft_states.entry(item.channel_id.clone()).or_insert_with(|| {
                    FftChannelState {
                        carry_buf: if carry_start < spec_input.len() {
                            spec_input[carry_start..].to_vec()
                        } else {
                            Vec::new()
                        },
                    }
                });
            }
        }

        backfill_channels.push(item.channel_id.clone());

        // Initialize raw context buffer with end of backfill data for live deconvolution
        if deconvolve {
            let deconv_context = (deconv_context_seconds * item.sample_rate) as usize;
            let tail_start = item.samples.len().saturating_sub(deconv_context);
            let mut ctx: VecDeque<f64> = item.samples[tail_start..].iter().cloned().collect();
            while ctx.len() > deconv_context {
                ctx.pop_front();
            }
            raw_context_bufs.insert(item.channel_id.clone(), ctx);
        }
    }

    // Send BackfillComplete
    let complete_msg = serde_json::json!({
        "type": "BackfillComplete",
        "data": { "channels": backfill_channels }
    });
    if sender.send(Message::Text(complete_msg.to_string())).await.is_err() {
        return;
    }
    debug!("Backfill complete for {} channels", backfill_channels.len());

    // Server-side ping keepalive every 30 seconds
    let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(30));
    ping_interval.tick().await;

    // Settings reactivity: check for changes every 2 seconds
    let mut settings_check_interval = tokio::time::interval(std::time::Duration::from_secs(2));
    settings_check_interval.tick().await;

    loop {
        tokio::select! {
            result = rx.recv() => {
                let msg = match result {
                    Ok(msg) => msg,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket client lagged, skipped {} messages. Clearing FFT carry buffers.", n);
                        // Clear carry buffers to avoid data discontinuity in spectrogram
                        for state in fft_states.values_mut() {
                            state.carry_buf.clear();
                        }
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                };
                match &msg {
                    WsMessage::Waveform { channel, timestamp, samples } => {
                        let timestamp_us = timestamp.timestamp_micros();
                        let sample_rate = {
                            let buffers = state.waveform_buffers.lock().unwrap();
                            buffers.get(channel).map(|b| b.sample_rate).unwrap_or(100.0)
                        };

                        // Deconvolve using FFT with context buffer.
                        // Produces two outputs:
                        //   deconv_samples: unfiltered deconvolved (for spectrogram)
                        //   filtered_samples: bandpass-filtered forward-only (for waveform display)
                        let (deconv_samples, filtered_samples) = if deconvolve {
                            if let Some(response) = resp_map.get(channel) {
                                if !response.poles.is_empty() {
                                    // Add raw samples to context buffer
                                    let deconv_context = (deconv_context_seconds * sample_rate) as usize;
                                    let raw_buf = raw_context_bufs.entry(channel.clone()).or_insert_with(VecDeque::new);
                                    for &s in samples.iter() {
                                        raw_buf.push_back(s);
                                    }
                                    while raw_buf.len() > deconv_context {
                                        raw_buf.pop_front();
                                    }

                                    // Deconvolve the full context buffer
                                    let buf_vec: Vec<f64> = raw_buf.iter().cloned().collect();
                                    let pre_filt = [0.1, 0.6, 0.95 * sample_rate, sample_rate];
                                    let mut all_deconv = deconvolve_response(&buf_vec, response, sample_rate, pre_filt, deconv_water_level);

                                    // Match rsudp: demean the deconvolved signal before filtering
                                    let vel_mean = all_deconv.iter().sum::<f64>() / all_deconv.len().max(1) as f64;
                                    for s in &mut all_deconv {
                                        *s -= vel_mean;
                                    }

                                    // Extract unfiltered tail for spectrogram
                                    let new_count = samples.len().min(all_deconv.len());
                                    let unfilt_tail = all_deconv[all_deconv.len() - new_count..].to_vec();

                                    // Apply forward-only bandpass filter to the full deconvolved window,
                                    // then extract filtered tail for waveform display.
                                    // This matches rsudp's actual behavior: obspy's filter() with default
                                    // zerophase=False uses sosfilt (forward-only), applied to the entire
                                    // accumulated stream with fresh filter state each update cycle.
                                    let filt_tail = if filter_enabled && filter_highpass > 0.0 && filter_lowpass > filter_highpass {
                                        let mut chain = BiquadChain::bandpass(filter_corners, filter_highpass, filter_lowpass, sample_rate);
                                        let all_filtered = chain.process_vec(&all_deconv);
                                        let n = new_count.min(all_filtered.len());
                                        all_filtered[all_filtered.len() - n..].to_vec()
                                    } else {
                                        unfilt_tail.clone()
                                    };

                                    // Apply extra divisor for GRAV units
                                    let (mut u, mut f) = (unfilt_tail, filt_tail);
                                    if units.to_uppercase() == "GRAV" {
                                        for s in &mut u { *s /= 9.81; }
                                        for s in &mut f { *s /= 9.81; }
                                    }
                                    (u, f)
                                } else {
                                    // No poles/zeros — use simple scalar division
                                    let sensitivity = response.sensitivity;
                                    let extra_divisor = if units.to_uppercase() == "GRAV" { 9.81 } else { 1.0 };
                                    let v: Vec<f64> = samples.iter().map(|&s| s / sensitivity / extra_divisor).collect();
                                    (v.clone(), v)
                                }
                            } else if let Some(&sensitivity) = sens_map.get(channel) {
                                let extra_divisor = if units.to_uppercase() == "GRAV" { 9.81 } else { 1.0 };
                                let v: Vec<f64> = samples.iter().map(|&s| s / sensitivity / extra_divisor).collect();
                                (v.clone(), v)
                            } else {
                                (samples.to_vec(), samples.to_vec())
                            }
                        } else {
                            (samples.to_vec(), samples.to_vec())
                        };

                        let waveform_packet = serialize_waveform_packet(
                            channel,
                            timestamp_us,
                            sample_rate as f32,
                            &filtered_samples,
                        );
                        if sender.send(Message::Binary(waveform_packet)).await.is_err() {
                            break;
                        }

                        // Spectrogram always uses deconvolved (unfiltered) data, matching rsudp default
                        // (rsudp's filter_spectrogram=False — avoids filter startup transient noise)
                        let spec_input: &[f64] = &deconv_samples;
                        let fft_state = fft_states.entry(channel.clone()).or_insert_with(FftChannelState::new);

                        // Record carry_buf length BEFORE processing to compute correct first-column timestamp.
                        // The carry_buf contains leftover samples from previous packets, so the first column
                        // in this batch actually starts carry_buf_len_before / sample_rate seconds BEFORE
                        // the current waveform packet's timestamp.
                        let carry_buf_len_before = fft_state.carry_buf.len();
                        let new_columns = compute_incremental_columns(fft_state, spec_input, &hann, &fft, sample_rate);

                        if !new_columns.is_empty() {
                            let hop_duration = HOP as f32 / sample_rate as f32;
                            let freq_bins = (NFFT / 2 + 1) as u16;
                            // Correct timestamp: account for carry_buf offset
                            let first_col_ts = timestamp_us - (carry_buf_len_before as f64 / sample_rate * 1_000_000.0) as i64;
                            let spec_packet = serialize_spectrogram_f32_packet(
                                channel,
                                first_col_ts,
                                sample_rate as f32,
                                hop_duration,
                                freq_bins,
                                &new_columns,
                            );
                            if sender.send(Message::Binary(spec_packet)).await.is_err() {
                                break;
                            }
                        }
                    },
                    WsMessage::Spectrogram { .. } => {
                        // Spectrogram messages are generated per-connection, not forwarded
                    },
                    _ => {
                        // Send text-based messages as JSON
                        if let Ok(json) = serde_json::to_string(&msg) {
                            if sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
            // Poll the WebSocket receiver to process control frames (Ping/Close)
            ws_msg = receiver.next() => {
                match ws_msg {
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        debug!("WebSocket client disconnected");
                        break;
                    }
                    _ => {}
                }
            }
            // Send periodic ping to keep connection alive
            _ = ping_interval.tick() => {
                if sender.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }
            // Check for settings changes (filter, spectrogram range)
            _ = settings_check_interval.tick() => {
                let (new_fe, new_hp, new_lp, new_fc, new_fmin, new_fmax) = {
                    let s = state.settings.read().unwrap();
                    (s.filter_waveform, s.filter_highpass, s.filter_lowpass,
                     s.filter_corners, s.spectrogram_freq_min, s.spectrogram_freq_max)
                };

                // Filter settings changed — update local state
                // (no persistent filter state to clear since we create fresh chains per deconv cycle)
                if new_fe != filter_enabled || new_hp != filter_highpass
                    || new_lp != filter_lowpass || new_fc != filter_corners
                {
                    filter_enabled = new_fe;
                    filter_highpass = new_hp;
                    filter_lowpass = new_lp;
                    filter_corners = new_fc;
                    // Also reset FFT states since input data characteristics changed
                    fft_states.clear();
                }

                // Update spectrogram freq range tracking (for future use)
                spec_freq_min = new_fmin;
                spec_freq_max = new_fmax;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T004: PSD normalization unit test — verify PSD values match matplotlib's default
    #[test]
    fn test_psd_normalization_sine_wave() {
        let sample_rate = 100.0;
        let nfft = 128;
        let freq = 10.0;
        let amplitude = 1000.0;

        let samples: Vec<f64> = (0..nfft)
            .map(|i| amplitude * (2.0 * std::f64::consts::PI * freq * i as f64 / sample_rate).sin())
            .collect();

        let window: Vec<f64> = (0..nfft)
            .map(|i| 0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / (nfft - 1) as f64).cos()))
            .collect();
        let window_power_sum: f64 = window.iter().map(|w| w * w).sum();
        let psd_norm = sample_rate * window_power_sum;

        let mean = samples.iter().sum::<f64>() / nfft as f64;
        let mut buffer: Vec<Complex<f64>> = samples.iter().zip(window.iter())
            .map(|(&s, &w)| Complex { re: (s - mean) * w, im: 0.0 })
            .collect();

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(nfft);
        fft.process(&mut buffer);

        let freq_bins = nfft / 2 + 1;
        let expected_bin = (freq * nfft as f64 / sample_rate).round() as usize;

        let c = &buffer[expected_bin];
        let mag_sq = c.re * c.re + c.im * c.im;
        let mut psd = mag_sq / psd_norm;
        psd *= 2.0; // one-sided correction
        let psd_db = 10.0 * psd.log10();

        // For sine amplitude=1000, PSD peak should be ~56 dB
        assert!(psd_db > 40.0, "PSD peak should be > 40 dB, got {:.1} dB", psd_db);
        assert!(psd_db < 70.0, "PSD peak should be < 70 dB, got {:.1} dB", psd_db);

        // Background bins should be much lower
        let off_bin = expected_bin + 5;
        if off_bin < freq_bins {
            let c_off = &buffer[off_bin];
            let mag_sq_off = c_off.re * c_off.re + c_off.im * c_off.im;
            let mut psd_off = mag_sq_off / psd_norm;
            psd_off *= 2.0;
            let psd_db_off = 10.0 * psd_off.max(1e-20).log10();
            assert!(psd_db - psd_db_off > 30.0,
                "Peak ({:.1} dB) should be >30 dB above background ({:.1} dB)", psd_db, psd_db_off);
        }
    }

    /// T005: Linear PSD power-law compression unit test
    #[test]
    fn test_linear_psd_compression_pipeline() {
        // Power-law compression: PSD^0.1 on linear values
        // Zero PSD → 0
        let compressed_zero = 0.0_f64.powf(0.1);
        assert_eq!(compressed_zero, 0.0, "Zero PSD^0.1 should be 0");

        // PSD = 1.0 → 1.0^0.1 = 1.0
        let compressed_one = 1.0_f64.powf(0.1);
        assert!((compressed_one - 1.0).abs() < 0.001);

        // PSD = 1e6 → 1e6^0.1 ≈ 3.981
        let compressed_strong = 1e6_f64.powf(0.1);
        assert!(compressed_strong > 3.9 && compressed_strong < 4.1,
            "1e6^0.1 should be ~3.981, got {}", compressed_strong);

        // PSD = 1e-10 → very small compressed value
        let compressed_tiny = 1e-10_f64.powf(0.1);
        assert!(compressed_tiny < 0.2, "1e-10^0.1 should be small, got {}", compressed_tiny);

        // u8 normalization: peak → 255, background → low
        let max_psd = 1e6_f64;
        let max_compressed = max_psd.powf(0.1);

        let u8_peak = ((max_psd.powf(0.1) / max_compressed).clamp(0.0, 1.0) * 255.0).round() as u8;
        assert_eq!(u8_peak, 255, "Peak should map to 255");

        let u8_bg = ((1e-10_f64.powf(0.1) / max_compressed).clamp(0.0, 1.0) * 255.0).round() as u8;
        assert!(u8_bg < 10, "Background should be very low, got {}", u8_bg);

        // Dynamic range: 1e-3 PSD should map to ~half brightness vs peak
        let mid_psd = 1e-3_f64;
        let mid_ratio = mid_psd.powf(0.1) / max_compressed;
        assert!(mid_ratio > 0.1 && mid_ratio < 0.5,
            "Mid PSD ratio should be moderate, got {:.3}", mid_ratio);
    }

    /// Test incremental columns pipeline: raw f32 compressed PSD values
    #[test]
    fn test_compute_incremental_columns_psd() {
        let sample_rate = 100.0;
        let hann = hanning_window(NFFT);
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(NFFT);
        let mut state = FftChannelState::new();

        // 2 seconds of 10 Hz sine
        let n_samples = (2.0 * sample_rate) as usize;
        let samples: Vec<f64> = (0..n_samples)
            .map(|i| 1000.0 * (2.0 * std::f64::consts::PI * 10.0 * i as f64 / sample_rate).sin())
            .collect();

        let columns = compute_incremental_columns(&mut state, &samples, &hann, &fft, sample_rate);
        assert!(!columns.is_empty(), "Should produce columns");

        for col in &columns {
            assert_eq!(col.len(), NFFT / 2 + 1);
        }

        // Peak bin (10 Hz → bin 13) should have high compressed PSD value
        let peak_bin = 13;
        let max_at_peak: f32 = columns.iter().map(|c| c[peak_bin]).fold(0.0_f32, f32::max);
        assert!(max_at_peak > 1.0, "Peak bin compressed PSD should be >1.0, got {:.3}", max_at_peak);

        // Background bins should have significantly lower compressed PSD
        let last_col = columns.last().unwrap();
        let peak_val = last_col[peak_bin];
        let bg_count = last_col.iter().enumerate()
            .filter(|(i, &v)| *i != peak_bin && v < peak_val * 0.5)
            .count();
        let bg_pct = bg_count as f64 / last_col.len() as f64;
        assert!(bg_pct > 0.5, "≥50% of bins should be significantly below peak, got {:.0}%", bg_pct * 100.0);
    }
}