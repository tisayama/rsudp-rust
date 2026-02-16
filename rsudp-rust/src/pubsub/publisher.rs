use std::collections::HashMap;
use std::time::Duration;
use google_cloud_pubsub::client::Client;
use google_cloud_pubsub::publisher::Publisher;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use prost::Message;
use tokio::sync::mpsc;
use tracing::{info, warn, error};

use crate::settings::PubsubSettings;
use super::dedup::generate_dedup_key;
use super::proto::{SeismicBatch, ChannelData};

/// Data sent from the pipeline to the publisher background task.
#[derive(Debug, Clone)]
pub struct SegmentData {
    pub channel: String,
    pub samples: Vec<i32>,
    pub start_time_ms: i64,
    pub sample_rate: f64,
}

/// Start the publisher background task. Returns an mpsc::Sender to feed segment data.
pub async fn start_publisher(
    client: &Client,
    config: &PubsubSettings,
    station: &str,
) -> Result<mpsc::Sender<SegmentData>, Box<dyn std::error::Error>> {
    let topic = client.topic(&config.topic);
    if !topic.exists(None).await? {
        warn!("pubsub: Topic '{}' does not exist. Creating it.", config.topic);
        topic.create(None, None).await?;
    }

    let publisher = topic.new_publisher(None);
    let (tx, rx) = mpsc::channel::<SegmentData>(256);
    let station = station.to_string();
    let batch_interval_ms = config.batch_interval_ms;

    tokio::spawn(async move {
        run_publisher_loop(publisher, rx, station, batch_interval_ms).await;
    });

    Ok(tx)
}

async fn run_publisher_loop(
    publisher: Publisher,
    mut rx: mpsc::Receiver<SegmentData>,
    station: String,
    batch_interval_ms: u64,
) {
    let mut buffer: HashMap<String, ChannelData> = HashMap::new();
    let mut window_start: Option<i64> = None;
    let mut sample_rate: f64 = 100.0;
    let mut interval = tokio::time::interval(Duration::from_millis(batch_interval_ms));

    loop {
        tokio::select! {
            seg = rx.recv() => {
                match seg {
                    Some(seg) => {
                        sample_rate = seg.sample_rate;
                        if window_start.is_none() {
                            let floored = (seg.start_time_ms / batch_interval_ms as i64) * batch_interval_ms as i64;
                            window_start = Some(floored);
                        }
                        let entry = buffer.entry(seg.channel.clone()).or_insert_with(|| {
                            ChannelData {
                                channel: seg.channel.clone(),
                                samples: Vec::new(),
                                start_time_ms: seg.start_time_ms,
                            }
                        });
                        entry.samples.extend_from_slice(&seg.samples);
                    }
                    None => {
                        // Channel closed, flush remaining and exit
                        if !buffer.is_empty() {
                            flush_batch(&publisher, &station, &mut buffer, &mut window_start, sample_rate, batch_interval_ms).await;
                        }
                        info!("pubsub: Publisher channel closed, exiting");
                        break;
                    }
                }
            }
            _ = interval.tick() => {
                if !buffer.is_empty() {
                    flush_batch(&publisher, &station, &mut buffer, &mut window_start, sample_rate, batch_interval_ms).await;
                }
            }
        }
    }
}

async fn flush_batch(
    publisher: &Publisher,
    station: &str,
    buffer: &mut HashMap<String, ChannelData>,
    window_start: &mut Option<i64>,
    sample_rate: f64,
    batch_interval_ms: u64,
) {
    let ws = match *window_start {
        Some(ws) => ws,
        None => return,
    };

    let channels: Vec<ChannelData> = buffer.drain().map(|(_, v)| v).collect();
    let total_samples: usize = channels.iter().map(|c| c.samples.len()).sum();

    let batch = SeismicBatch {
        station: station.to_string(),
        window_start_ms: ws,
        window_end_ms: ws + batch_interval_ms as i64,
        sample_rate,
        channels,
    };

    let dedup_key = generate_dedup_key(station, ws);
    let window_start_str = chrono::DateTime::from_timestamp_millis(ws)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
        .unwrap_or_default();

    let data = batch.encode_to_vec();

    // Validate message size (10MB limit)
    if data.len() > 9_000_000 {
        warn!("pubsub: Message size ({} bytes) approaching 10MB limit!", data.len());
    }

    let mut attributes = HashMap::new();
    attributes.insert("dedup_key".to_string(), dedup_key.clone());
    attributes.insert("station".to_string(), station.to_string());
    attributes.insert("window_start".to_string(), window_start_str);

    let msg = PubsubMessage {
        data,
        attributes,
        ordering_key: station.to_string(),
        ..Default::default()
    };

    let max_retries = 3u32;
    let mut last_error = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            let backoff = Duration::from_secs(1 << (attempt - 1)); // 1s, 2s, 4s
            warn!("pubsub: Retrying publish (attempt {}/{}) after {:?}", attempt, max_retries, backoff);
            tokio::time::sleep(backoff).await;
        }

        match publisher.publish(msg.clone()).await.get().await {
            Ok(_) => {
                if attempt > 0 {
                    info!("pubsub: Publish succeeded on retry {}", attempt);
                }
                info!("pubsub: Published batch: station={}, channels={}, samples={}, dedup_key={}",
                    station, batch.channels.len(), total_samples, dedup_key);
                last_error = None;
                break;
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
    }

    if let Some(e) = last_error {
        error!("pubsub: Failed to publish after {} retries: {}", max_retries, e);
    }

    *window_start = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_data_creation() {
        let seg = SegmentData {
            channel: "EHZ".to_string(),
            samples: vec![100, 200, 300],
            start_time_ms: 1732525283500,
            sample_rate: 100.0,
        };
        assert_eq!(seg.channel, "EHZ");
        assert_eq!(seg.samples.len(), 3);
    }

    #[test]
    fn test_batch_encoding() {
        let batch = SeismicBatch {
            station: "AM.R6E01".to_string(),
            window_start_ms: 1732525283500,
            window_end_ms: 1732525284000,
            sample_rate: 100.0,
            channels: vec![
                ChannelData {
                    channel: "EHZ".to_string(),
                    samples: vec![100, -200, 300],
                    start_time_ms: 1732525283500,
                },
                ChannelData {
                    channel: "ENE".to_string(),
                    samples: vec![50, 60, 70],
                    start_time_ms: 1732525283500,
                },
            ],
        };

        let encoded = batch.encode_to_vec();
        assert!(!encoded.is_empty());

        let decoded = SeismicBatch::decode(&encoded[..]).unwrap();
        assert_eq!(decoded.station, "AM.R6E01");
        assert_eq!(decoded.channels.len(), 2);
        assert_eq!(decoded.channels[0].samples, vec![100, -200, 300]);
        assert_eq!(decoded.sample_rate, 100.0);
    }
}
