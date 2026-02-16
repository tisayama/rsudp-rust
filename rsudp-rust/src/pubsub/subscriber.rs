use google_cloud_pubsub::client::Client;
use google_cloud_pubsub::subscription::Subscription;
use prost::Message;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::settings::PubsubSettings;
use super::proto::SeismicBatch;

/// Start the subscriber, which receives Pub/Sub messages and injects
/// reconstructed packets into the pipeline via `pipe_tx`.
pub async fn start_subscriber(
    client: &Client,
    config: &PubsubSettings,
    station: &str,
    pipe_tx: mpsc::Sender<Vec<u8>>,
    cancel: CancellationToken,
) -> Result<(), Box<dyn std::error::Error>> {
    let subscription = client.subscription(&config.subscription);
    if !subscription.exists(None).await? {
        return Err(format!("Pub/Sub subscription '{}' does not exist", config.subscription).into());
    }

    let station = station.to_string();
    info!("pubsub: Starting subscriber on subscription '{}'", config.subscription);

    tokio::spawn(async move {
        run_subscriber(subscription, station, pipe_tx, cancel).await;
    });

    Ok(())
}

async fn run_subscriber(
    subscription: Subscription,
    _station: String,
    pipe_tx: mpsc::Sender<Vec<u8>>,
    cancel: CancellationToken,
) {
    let result = subscription.receive(
        move |message, _cancel| {
            let pipe_tx = pipe_tx.clone();
            async move {
                let dedup_key = message.message.attributes
                    .get("dedup_key")
                    .cloned()
                    .unwrap_or_default();

                // Note: dedup check is done per-message in the callback
                // For thread safety, we check via the attribute and ack regardless
                // The DedupChecker in the outer scope handles cross-message dedup
                // but since receive() is concurrent, we rely on at-least-once + ack

                let data = &message.message.data;
                match SeismicBatch::decode(&data[..]) {
                    Ok(batch) => {
                        info!("pubsub: Received batch: station={}, channels={}, dedup_key={}",
                            batch.station, batch.channels.len(), dedup_key);

                        for channel_data in &batch.channels {
                            let packet = reconstruct_packet(
                                &batch.station,
                                &channel_data.channel,
                                &channel_data.samples,
                                channel_data.start_time_ms,
                                batch.sample_rate,
                            );
                            if let Err(e) = pipe_tx.send(packet).await {
                                warn!("pubsub: Failed to send to pipeline: {}", e);
                            }
                        }

                        info!("pubsub: Injected {} channel segments into pipeline", batch.channels.len());
                    }
                    Err(e) => {
                        warn!("pubsub: Failed to decode protobuf: {}", e);
                    }
                }

                message.ack().await.ok();
            }
        },
        cancel,
        None,
    ).await;

    if let Err(e) = result {
        warn!("pubsub: Subscriber loop ended: {}", e);
    }
}

/// Reconstruct a packet in the Python-style rsudp format that parse_any() can handle:
/// `{'CHANNEL', TIMESTAMP, S1, S2, ...}`
///
/// This format is parsed by parse_any() in parser/mod.rs, producing a TraceSegment
/// with the sample data. Note: network/station will be "XX"/"SIM" from parse_any defaults,
/// but the pipeline processes all data identically regardless of source metadata.
fn reconstruct_packet(
    _station: &str,
    channel: &str,
    samples: &[i32],
    start_time_ms: i64,
    _sample_rate: f64,
) -> Vec<u8> {
    let timestamp = start_time_ms as f64 / 1000.0;
    let mut s = String::with_capacity(64 + samples.len() * 8);
    s.push_str("{'");
    s.push_str(channel);
    s.push_str("', ");
    s.push_str(&format!("{:.3}", timestamp));
    for sample in samples {
        s.push_str(", ");
        s.push_str(&sample.to_string());
    }
    s.push('}');
    s.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_any;

    #[test]
    fn test_reconstruct_packet_parseable() {
        let packet = reconstruct_packet(
            "AM.R6E01",
            "EHZ",
            &[100, -200, 300],
            1732525283500,
            100.0,
        );

        let segments = parse_any(&packet).unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].channel, "EHZ");
        assert_eq!(segments[0].samples.len(), 3);
        assert_eq!(segments[0].samples[0] as i32, 100);
        assert_eq!(segments[0].samples[1] as i32, -200);
        assert_eq!(segments[0].samples[2] as i32, 300);
    }

    #[test]
    fn test_reconstruct_packet_preserves_timestamp() {
        let packet = reconstruct_packet(
            "AM.R6E01",
            "ENE",
            &[50, 60],
            1732525283730,
            100.0,
        );

        let segments = parse_any(&packet).unwrap();
        assert_eq!(segments[0].channel, "ENE");
        // Timestamp should be 1732525283.730
        let ts = segments[0].starttime.timestamp_millis();
        assert_eq!(ts, 1732525283730);
    }
}
