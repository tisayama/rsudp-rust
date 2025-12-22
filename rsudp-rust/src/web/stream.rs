use crate::intensity::IntensityResult;
use crate::trigger::AlertEvent;
use crate::web::history::{AlertHistoryManager, SharedHistory};
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
use tracing::{info, warn, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSettings {
    pub scale: f64,
    pub window_seconds: f64,
    pub save_pct: f64,
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

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<WebState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: axum::extract::ws::WebSocket, state: WebState) {
    let (mut sender, _) = socket.split();
    let mut rx = state.subscribe();

    while let Ok(msg) = rx.recv().await {
        if let Ok(json) = serde_json::to_string(&msg)
            && sender.send(Message::Text(json)).await.is_err()
        {
            break;
        }
    }
}