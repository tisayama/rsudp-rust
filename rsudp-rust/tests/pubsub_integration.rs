//! Integration tests for Pub/Sub publish-subscribe cycle.
//! Requires PUBSUB_EMULATOR_HOST to be set (e.g., "localhost:8085").
//!
//! Start the emulator:
//!   docker compose -f docker-compose.test.yml up -d
//!
//! Run tests:
//!   PUBSUB_EMULATOR_HOST=localhost:8085 cargo test --test pubsub_integration

use google_cloud_pubsub::client::{Client, ClientConfig};
use google_cloud_pubsub::subscription::SubscriptionConfig;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_gax::conn::Environment;
use prost::Message;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use rsudp_rust::pubsub::proto::{SeismicBatch, ChannelData};
use rsudp_rust::pubsub::dedup::{generate_dedup_key, DedupChecker};

/// Skip test if emulator is not running.
macro_rules! require_emulator {
    () => {
        match std::env::var("PUBSUB_EMULATOR_HOST") {
            Ok(val) if !val.is_empty() => {}
            _ => {
                eprintln!("Skipping: PUBSUB_EMULATOR_HOST not set");
                return;
            }
        }
    };
}

async fn create_emulator_client() -> Client {
    let host = std::env::var("PUBSUB_EMULATOR_HOST").unwrap();
    let config = ClientConfig {
        environment: Environment::Emulator(host),
        project_id: Some("test-project".to_string()),
        ..Default::default()
    };
    Client::new(config).await.expect("Failed to create emulator client")
}

fn unique_name(prefix: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let short = &id[..8];
    format!("{}-{}", prefix, short)
}

fn make_test_batch(
    station: &str,
    channels: Vec<(&str, Vec<i32>)>,
    window_start_ms: i64,
) -> SeismicBatch {
    SeismicBatch {
        station: station.to_string(),
        window_start_ms,
        window_end_ms: window_start_ms + 500,
        sample_rate: 100.0,
        channels: channels
            .into_iter()
            .map(|(ch, samples)| ChannelData {
                channel: ch.to_string(),
                samples,
                start_time_ms: window_start_ms,
            })
            .collect(),
    }
}

async fn publish_batch(client: &Client, topic_name: &str, batch: &SeismicBatch) {
    let topic = client.topic(topic_name);
    let publisher = topic.new_publisher(None);

    let data = batch.encode_to_vec();
    let dedup_key = generate_dedup_key(&batch.station, batch.window_start_ms);

    let mut attributes = HashMap::new();
    attributes.insert("dedup_key".to_string(), dedup_key);
    attributes.insert("station".to_string(), batch.station.clone());

    let msg = PubsubMessage {
        data: data.into(),
        attributes,
        ordering_key: batch.station.clone(),
        ..Default::default()
    };

    publisher.publish(msg).await.get().await.expect("Publish failed");
}

async fn setup_topic_and_subscription(
    client: &Client,
    topic_name: &str,
    sub_name: &str,
    enable_ordering: bool,
) {
    let topic = client.topic(topic_name);
    topic.create(None, None).await.expect("Topic creation failed");

    let cfg = SubscriptionConfig {
        enable_message_ordering: enable_ordering,
        ..Default::default()
    };
    client
        .create_subscription(sub_name, topic_name, cfg, None)
        .await
        .expect("Subscription creation failed");
}

/// Receive data from a subscription, collecting up to `expected` messages
/// within the given timeout.
struct TestReceiver {
    rx: mpsc::Receiver<(Vec<u8>, HashMap<String, String>)>,
    cancel: CancellationToken,
}

impl TestReceiver {
    fn start(client: &Client, sub_name: &str) -> Self {
        let (tx, rx) = mpsc::channel(64);
        let cancel = CancellationToken::new();
        let subscription = client.subscription(sub_name);
        let cancel_recv = cancel.clone();

        tokio::spawn(async move {
            subscription
                .receive(
                    move |msg, _cancel| {
                        let tx = tx.clone();
                        async move {
                            let data = msg.message.data.to_vec();
                            let attrs = msg.message.attributes.clone();
                            tx.send((data, attrs)).await.ok();
                            msg.ack().await.ok();
                        }
                    },
                    cancel_recv,
                    None,
                )
                .await
                .ok();
        });

        Self { rx, cancel }
    }

    async fn recv_one(&mut self, timeout_secs: u64) -> Option<(Vec<u8>, HashMap<String, String>)> {
        tokio::time::timeout(Duration::from_secs(timeout_secs), self.rx.recv())
            .await
            .ok()
            .flatten()
    }

    async fn recv_many(
        &mut self,
        count: usize,
        timeout_secs: u64,
    ) -> Vec<(Vec<u8>, HashMap<String, String>)> {
        let mut results = Vec::new();
        for _ in 0..count {
            match tokio::time::timeout(Duration::from_secs(timeout_secs), self.rx.recv()).await {
                Ok(Some(item)) => results.push(item),
                _ => break,
            }
        }
        results
    }

    fn stop(self) {
        self.cancel.cancel();
    }
}

// --- T031: Publish-subscribe roundtrip ---

#[tokio::test]
async fn test_pubsub_roundtrip() {
    require_emulator!();

    let client = create_emulator_client().await;
    let topic_name = unique_name("roundtrip");
    let sub_name = unique_name("roundtrip-sub");

    setup_topic_and_subscription(&client, &topic_name, &sub_name, true).await;

    // Publish test batch with multiple channels
    let batch = make_test_batch(
        "AM.R6E01",
        vec![("EHZ", vec![100, -200, 300]), ("ENE", vec![50, 60, 70])],
        1732525283500,
    );
    publish_batch(&client, &topic_name, &batch).await;

    // Receive
    let mut receiver = TestReceiver::start(&client, &sub_name);
    let (data, attrs) = receiver.recv_one(10).await.expect("No message received");
    receiver.stop();

    // Verify protobuf roundtrip
    let received = SeismicBatch::decode(&data[..]).expect("Decode failed");
    assert_eq!(received.station, "AM.R6E01");
    assert_eq!(received.channels.len(), 2);
    assert_eq!(received.sample_rate, 100.0);
    assert_eq!(received.window_start_ms, 1732525283500);
    assert_eq!(received.window_end_ms, 1732525284000);

    // Check channels are present (order may vary due to HashMap drain in publisher)
    let ch_names: Vec<&str> = received
        .channels
        .iter()
        .map(|c| c.channel.as_str())
        .collect();
    assert!(ch_names.contains(&"EHZ"), "Missing EHZ channel");
    assert!(ch_names.contains(&"ENE"), "Missing ENE channel");

    // Verify EHZ samples
    let ehz = received
        .channels
        .iter()
        .find(|c| c.channel == "EHZ")
        .unwrap();
    assert_eq!(ehz.samples, vec![100, -200, 300]);

    // Verify ENE samples
    let ene = received
        .channels
        .iter()
        .find(|c| c.channel == "ENE")
        .unwrap();
    assert_eq!(ene.samples, vec![50, 60, 70]);

    // Verify attributes
    assert!(attrs.contains_key("dedup_key"));
    assert_eq!(attrs.get("station").unwrap(), "AM.R6E01");
}

// --- T032: Deduplication integration test ---

#[tokio::test]
async fn test_pubsub_dedup() {
    require_emulator!();

    let client = create_emulator_client().await;
    let topic_name = unique_name("dedup");
    let sub_name = unique_name("dedup-sub");

    setup_topic_and_subscription(&client, &topic_name, &sub_name, false).await;

    // Publish two messages in the SAME 500ms window (same dedup_key)
    let batch1 = make_test_batch("AM.R6E01", vec![("EHZ", vec![100, 200])], 1732525283500);
    let batch2 = make_test_batch("AM.R6E01", vec![("EHZ", vec![300, 400])], 1732525283600);

    publish_batch(&client, &topic_name, &batch1).await;
    publish_batch(&client, &topic_name, &batch2).await;

    // Both messages arrive from Pub/Sub (it doesn't deduplicate)
    let mut receiver = TestReceiver::start(&client, &sub_name);
    let messages = receiver.recv_many(2, 10).await;
    receiver.stop();

    assert_eq!(
        messages.len(),
        2,
        "Pub/Sub should deliver both messages (no server-side dedup)"
    );

    // Apply subscriber-side DedupChecker
    let mut dedup = DedupChecker::new(100);
    let mut unique_count = 0;
    for (_, attrs) in &messages {
        let key = attrs.get("dedup_key").cloned().unwrap_or_default();
        if dedup.check_and_insert(&key) {
            unique_count += 1;
        }
    }

    assert_eq!(
        unique_count, 1,
        "DedupChecker should filter duplicate dedup_keys within the same 500ms window"
    );
}

// --- T033: Ordering integration test ---

#[tokio::test]
async fn test_pubsub_ordering() {
    require_emulator!();

    let client = create_emulator_client().await;
    let topic_name = unique_name("ordering");
    let sub_name = unique_name("ordering-sub");

    setup_topic_and_subscription(&client, &topic_name, &sub_name, true).await;

    // Publish 3 batches with sequential windows and same ordering_key
    let windows: Vec<i64> = vec![1732525283000, 1732525283500, 1732525284000];
    for (i, &w) in windows.iter().enumerate() {
        let batch = make_test_batch(
            "AM.R6E01",
            vec![("EHZ", vec![(i + 1) as i32 * 100])],
            w,
        );
        publish_batch(&client, &topic_name, &batch).await;
    }

    // Receive
    let mut receiver = TestReceiver::start(&client, &sub_name);
    let messages = receiver.recv_many(3, 10).await;
    receiver.stop();

    assert_eq!(messages.len(), 3, "Should receive all 3 messages");

    // Verify ordering by window_start_ms
    let received_windows: Vec<i64> = messages
        .iter()
        .map(|(data, _)| {
            let batch = SeismicBatch::decode(&data[..]).unwrap();
            batch.window_start_ms
        })
        .collect();

    assert_eq!(
        received_windows, windows,
        "Messages should arrive in publish order (same ordering_key)"
    );
}
