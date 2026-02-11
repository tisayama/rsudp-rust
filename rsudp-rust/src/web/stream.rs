use crate::intensity::IntensityResult;
use crate::trigger::AlertEvent;
use crate::web::history::{AlertHistoryManager, SharedHistory};
use crate::web::plot::{compute_spectrogram, SpectrogramU8};
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
        spectrogram: SpectrogramU8,
    },
}

#[derive(Clone)]
pub struct WebState {
    pub tx: broadcast::Sender<WsMessage>,
    pub settings: Arc<RwLock<PlotSettings>>,
    pub history: SharedHistory,
    pub waveform_buffers: Arc<Mutex<HashMap<String, ChannelBuffer>>>, // Updated type
    pub alert_max_intensities: Arc<Mutex<HashMap<uuid::Uuid, f64>>>,
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
            })),
            history: Arc::new(Mutex::new(AlertHistoryManager::new())),
            waveform_buffers: Arc::new(Mutex::new(HashMap::new())),
            alert_max_intensities: Arc::new(Mutex::new(HashMap::new())),
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

/// Serialize a spectrogram packet as binary (type 0x03)
/// Format: [0x03][channelIdLen:u8][channelId:utf8][timestamp:i64le(μs)][sampleRate:f32le][frequencyBins:u16le][columnsCount:u16le][data:u8[columnsCount × frequencyBins]]
pub fn serialize_spectrogram_packet(
    channel_id: &str,
    timestamp_us: i64,
    sample_rate: f32,
    hop_duration: f32,
    spec: &SpectrogramU8,
) -> Vec<u8> {
    let channel_bytes = channel_id.as_bytes();
    let freq_bins = spec.frequency_bins as u16;
    let cols_count = spec.columns.len() as u16;
    let data_size = (cols_count as usize) * (freq_bins as usize);

    // Header: 1 + 1 + channelIdLen + 8 + 4 + 4 + 2 + 2 = 22 + channelIdLen
    let total_size = 1 + 1 + channel_bytes.len() + 8 + 4 + 4 + 2 + 2 + data_size;
    let mut buf = Vec::with_capacity(total_size);

    buf.push(0x03); // type
    buf.push(channel_bytes.len() as u8); // channelIdLen
    buf.extend_from_slice(channel_bytes); // channelId
    buf.extend_from_slice(&timestamp_us.to_le_bytes()); // timestamp i64le
    buf.extend_from_slice(&sample_rate.to_le_bytes()); // sampleRate f32le
    buf.extend_from_slice(&hop_duration.to_le_bytes()); // hopDuration f32le
    buf.extend_from_slice(&freq_bins.to_le_bytes()); // frequencyBins u16le
    buf.extend_from_slice(&cols_count.to_le_bytes()); // columnsCount u16le

    // Data: column-major layout
    for col in &spec.columns {
        buf.extend_from_slice(col);
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
const NOVERLAP: usize = NFFT - HOP; // 115

/// Hanning window coefficients
fn hanning_window(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| 0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / (n - 1) as f64).cos()))
        .collect()
}

/// Per-channel state for incremental FFT computation
struct FftChannelState {
    carry_buf: Vec<f64>,
    running_max: f64,
}

impl FftChannelState {
    fn new() -> Self {
        Self {
            carry_buf: Vec::new(),
            running_max: 1e-10,
        }
    }
}

/// Compute new spectrogram columns incrementally from carry buffer + new samples.
/// Returns u8-normalized columns. Updates state.carry_buf and state.running_max.
fn compute_incremental_columns(
    state: &mut FftChannelState,
    new_samples: &[f64],
    hann: &[f64],
    fft: &std::sync::Arc<dyn rustfft::Fft<f64>>,
) -> Vec<Vec<u8>> {
    let freq_bins = NFFT / 2 + 1;

    // Combine carry buffer with new samples
    let mut combined = Vec::with_capacity(state.carry_buf.len() + new_samples.len());
    combined.extend_from_slice(&state.carry_buf);
    combined.extend_from_slice(new_samples);

    if combined.len() < NFFT {
        state.carry_buf = combined;
        return Vec::new();
    }

    // Compute FFT windows
    let mut raw_columns: Vec<Vec<f64>> = Vec::new();
    let mut batch_max: f64 = 0.0;
    let mut pos = 0;

    while pos + NFFT <= combined.len() {
        let chunk = &combined[pos..pos + NFFT];
        let mean = chunk.iter().sum::<f64>() / NFFT as f64;

        let mut buffer: Vec<Complex<f64>> = chunk.iter().zip(hann.iter())
            .map(|(&s, &w)| Complex { re: (s - mean) * w, im: 0.0 })
            .collect();

        fft.process(&mut buffer);

        let mags: Vec<f64> = buffer.iter().take(freq_bins)
            .map(|c| c.re * c.re + c.im * c.im)
            .collect();

        for &m in &mags {
            if m > batch_max { batch_max = m; }
        }

        raw_columns.push(mags);
        pos += HOP;
    }

    // Save unprocessed samples as carry for next batch
    state.carry_buf = combined[pos..].to_vec();

    if raw_columns.is_empty() {
        return Vec::new();
    }

    // Update running max: slow decay (~30s half-life at ~7.7 cols/sec)
    let decay = 0.997_f64.powi(raw_columns.len() as i32);
    state.running_max *= decay;
    if state.running_max < 1e-10 { state.running_max = 1e-10; }
    if batch_max > state.running_max {
        state.running_max = batch_max;
    }

    // Normalize to u8
    let norm_max = state.running_max;
    raw_columns.iter().map(|mags| {
        mags.iter().map(|&mag_sq| {
            let normalized = (mag_sq / norm_max).powf(0.1);
            (normalized * 255.0).round().min(255.0).max(0.0) as u8
        }).collect()
    }).collect()
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
        let waveform_packet = serialize_waveform_packet(
            &item.channel_id,
            item.timestamp_us,
            item.sample_rate as f32,
            &item.samples,
        );
        if sender.send(Message::Binary(waveform_packet)).await.is_err() {
            return;
        }

        if item.samples.len() >= NFFT {
            let hop_duration = HOP as f32 / item.sample_rate as f32;

            // Compute raw spectrogram to get global max for normalization
            let raw_spec = compute_spectrogram(&item.samples, item.sample_rate, NFFT, NOVERLAP);

            if !raw_spec.data.is_empty() {
                // Find global max across all columns
                let mut max_mag_sq: f64 = 1e-10;
                for row in &raw_spec.data {
                    for &val in row {
                        if val > max_mag_sq { max_mag_sq = val; }
                    }
                }

                // Normalize to u8 using global max (same as compute_spectrogram_u8)
                let columns: Vec<Vec<u8>> = raw_spec.data.iter().map(|row| {
                    row.iter().map(|&mag_sq| {
                        let normalized = (mag_sq / max_mag_sq).powf(0.1);
                        (normalized * 255.0).round().min(255.0).max(0.0) as u8
                    }).collect()
                }).collect();

                let spec = SpectrogramU8 {
                    frequency_bins: NFFT / 2 + 1,
                    sample_rate: item.sample_rate,
                    columns,
                    timestamps: raw_spec.times,
                };

                let spec_packet = serialize_spectrogram_packet(
                    &item.channel_id,
                    item.timestamp_us,
                    item.sample_rate as f32,
                    hop_duration,
                    &spec,
                );
                if sender.send(Message::Binary(spec_packet)).await.is_err() {
                    return;
                }

                // Initialize FFT state with carry buffer AND backfill's global max
                // This bridges the normalization between backfill and live data
                let num_cols = raw_spec.data.len();
                let carry_start = num_cols * HOP;
                fft_states.entry(item.channel_id.clone()).or_insert_with(|| {
                    FftChannelState {
                        carry_buf: if carry_start < item.samples.len() {
                            item.samples[carry_start..].to_vec()
                        } else {
                            Vec::new()
                        },
                        running_max: max_mag_sq,
                    }
                });
            }
        }

        backfill_channels.push(item.channel_id.clone());
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

    while let Ok(msg) = rx.recv().await {
        match &msg {
            WsMessage::Waveform { channel, timestamp, samples } => {
                let timestamp_us = timestamp.timestamp_micros();
                let sample_rate = {
                    let buffers = state.waveform_buffers.lock().unwrap();
                    buffers.get(channel).map(|b| b.sample_rate).unwrap_or(100.0)
                };

                // Send waveform binary packet
                let waveform_packet = serialize_waveform_packet(
                    channel,
                    timestamp_us,
                    sample_rate as f32,
                    samples,
                );
                if sender.send(Message::Binary(waveform_packet)).await.is_err() {
                    break;
                }

                // Compute incremental spectrogram columns (sliding window)
                let fft_state = fft_states.entry(channel.clone()).or_insert_with(FftChannelState::new);
                let new_columns = compute_incremental_columns(fft_state, samples, &hann, &fft);

                if !new_columns.is_empty() {
                    let hop_duration = HOP as f32 / sample_rate as f32;
                    let new_spec = SpectrogramU8 {
                        frequency_bins: NFFT / 2 + 1,
                        sample_rate,
                        columns: new_columns,
                        timestamps: Vec::new(),
                    };
                    let spec_packet = serialize_spectrogram_packet(
                        channel,
                        timestamp_us,
                        sample_rate as f32,
                        hop_duration,
                        &new_spec,
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
}