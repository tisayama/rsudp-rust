use std::collections::HashMap;
use crate::trigger::{AlertManager, AlertConfig};
use crate::parser::{TraceSegment, mseed::parse_mseed_record};
use crate::web::{WaveformPacket, stream::WsMessage};
use tokio::sync::{mpsc, broadcast};
use tracing::{info, error};

pub struct PipelineManager {
    alerts: HashMap<String, AlertManager>,
    ws_tx: broadcast::Sender<WsMessage>,
}

impl PipelineManager {
    pub fn new(ws_tx: broadcast::Sender<WsMessage>) -> Self {
        Self {
            alerts: HashMap::new(),
            ws_tx,
        }
    }

    pub async fn process_segment(&mut self, segment: TraceSegment) {
        let nslc = segment.nslc();
        
        // Broadcast raw waveform to WebUI
        let packet = WaveformPacket {
            channel_id: nslc.clone(),
            timestamp: segment.starttime,
            samples: segment.samples.iter().map(|&s| s as f32).collect(),
            sample_rate: segment.sampling_rate as f32,
        };
        let _ = self.ws_tx.send(WsMessage::Waveform(packet));

        // Get or create AlertManager for this channel
        if !self.alerts.contains_key(&nslc) {
            let (event_tx, mut event_rx) = mpsc::channel(10);
            let config = AlertConfig {
                sta_seconds: 1.0,
                lta_seconds: 30.0,
                threshold: 3.0,
                reset_threshold: 1.5,
                min_duration: 0.0,
                channel_id: nslc.clone(),
                filter_config: None,
                sample_rate: segment.sampling_rate,
            };
            let manager = AlertManager::new(config, event_tx);
            self.alerts.insert(nslc.clone(), manager);

            // Spawn task to forward alerts to WebUI
            let ws_tx_inner = self.ws_tx.clone();
            tokio::spawn(async move {
                while let Some(event) = event_rx.recv().await {
                    let _ = ws_tx_inner.send(WsMessage::Alert(event));
                }
            });
        }

        if let Some(manager) = self.alerts.get_mut(&nslc) {
            let mut current_time = segment.starttime;
            let dt = chrono::Duration::nanoseconds((1e9 / segment.sampling_rate) as i64);
            
            for sample in segment.samples {
                if let Err(e) = manager.process_sample(sample, current_time).await {
                    error!("AlertManager error for {}: {}", nslc, e);
                }
                current_time = current_time + dt;
            }
        }
    }
}

pub async fn run_pipeline(
    mut input_rx: mpsc::Receiver<Vec<u8>>,
    mut manager: PipelineManager,
) {
    info!("Pipeline started");
    while let Some(data) = input_rx.recv().await {
        match parse_mseed_record(&data) {
            Ok(segments) => {
                for segment in segments {
                    manager.process_segment(segment).await;
                }
            }
            Err(e) => {
                error!("Parser error: {}", e);
            }
        }
    }
}