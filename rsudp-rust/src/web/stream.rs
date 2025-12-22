use crate::intensity::IntensityResult;
use crate::trigger::AlertEvent;
use crate::web::history::{AlertHistoryManager, SharedHistory};
use axum::{
    extract::ws::Message,
    extract::{State, WebSocketUpgrade},
    response::Response,
};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::broadcast;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSettings {
    pub scale: f64,
    pub window_seconds: f64,
    pub save_pct: f64,
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
    pub waveform_buffers: Arc<Mutex<HashMap<String, VecDeque<f64>>>>,
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
