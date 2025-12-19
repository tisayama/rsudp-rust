use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::intensity::IntensityResult;
use crate::trigger::AlertEvent;
use axum::{extract::{State, WebSocketUpgrade}, response::Response, extract::ws::Message};
use std::sync::{Arc, RwLock};
use futures_util::{SinkExt, StreamExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSettings {
    pub scale: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    Waveform { channel: String, timestamp: DateTime<Utc>, samples: Vec<f64> },
    Alert(AlertEvent),
    Intensity(IntensityResult),
}

#[derive(Clone)]
pub struct WebState {
    pub tx: broadcast::Sender<WsMessage>,
    pub settings: Arc<RwLock<PlotSettings>>,
}

impl WebState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { 
            tx,
            settings: Arc::new(RwLock::new(PlotSettings { scale: 1.0 })),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WsMessage> {
        self.tx.subscribe()
    }

    pub async fn broadcast_alert(&self, ev: AlertEvent) {
        let _ = self.tx.send(WsMessage::Alert(ev));
    }

    pub async fn broadcast_waveform(&self, channel: String, timestamp: DateTime<Utc>, samples: Vec<f64>) {
        let _ = self.tx.send(WsMessage::Waveform { channel, timestamp, samples });
    }

    pub async fn broadcast_intensity(&self, res: IntensityResult) {
        let _ = self.tx.send(WsMessage::Intensity(res));
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: axum::extract::ws::WebSocket, state: WebState) {
    let (mut sender, _) = socket.split();
    let mut rx = state.subscribe();

    while let Ok(msg) = rx.recv().await {
        if let Ok(json) = serde_json::to_string(&msg) {
            if sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    }
}
