use tokio::sync::broadcast;
use crate::web::{WaveformPacket, stream::WsMessage};
use chrono::Utc;
use std::time::Duration;

pub async fn start_mock_producer(tx: broadcast::Sender<WsMessage>) {
    let mut interval = tokio::time::interval(Duration::from_millis(100)); // 10Hz batches
    let channels = vec!["SHZ", "EHZ", "ENE", "ENN", "ENZ", "HDF"];
    let sample_rate = 100.0;
    let mut count = 0;

    loop {
        interval.tick().await;
        for &ch in &channels {
            let mut samples = Vec::with_capacity(10);
            for i in 0..10 {
                let val = ( (count + i) as f32 / 10.0 ).sin() * 1000.0;
                samples.push(val);
            }
            
            let packet = WaveformPacket {
                channel_id: ch.to_string(),
                timestamp: Utc::now(),
                samples,
                sample_rate,
            };
            
            let _ = tx.send(WsMessage::Waveform(packet));
        }
        count += 10;
    }
}
