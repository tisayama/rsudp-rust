use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
    response::Response,
    Extension,
};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use futures_util::{StreamExt, SinkExt};
use crate::web::{WaveformPacket, PlotSettings, AlertEvent};

pub struct WebState {
    pub settings: Arc<RwLock<PlotSettings>>,
    pub tx: broadcast::Sender<WsMessage>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    Waveform(WaveformPacket),
    Alert(AlertEvent),
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<WebState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<WebState>) {
    let (mut sender, mut _receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            match msg {
                WsMessage::Waveform(packet) => {
                    if sender.send(Message::Binary(packet.to_binary())).await.is_err() {
                        break;
                    }
                }
                WsMessage::Alert(alert) => {
                    let json = serde_json::to_string(&alert).unwrap();
                    // Prepend type byte 1 for Alert
                    let mut buf = vec![1];
                    buf.extend_from_slice(json.as_bytes());
                    if sender.send(Message::Binary(buf)).await.is_err() {
                        break;
                    }
                }
            }
        }
    });
}