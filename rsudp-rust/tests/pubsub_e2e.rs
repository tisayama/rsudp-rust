//! End-to-end test: streamer → publisher rsudp → Pub/Sub emulator → subscriber rsudp → pipeline
//!
//! Requires:
//!   - PUBSUB_EMULATOR_HOST to be set (e.g., "localhost:8085")
//!   - Pub/Sub emulator running (docker compose -f docker-compose.test.yml up -d)
//!   - MiniSEED test data at ../references/mseed/fdsnws.mseed
//!
//! Run:
//!   PUBSUB_EMULATOR_HOST=localhost:8085 cargo test --test pubsub_e2e -- --nocapture

use std::fs;
use std::io::Read;
use std::net::UdpSocket;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempdir;

use google_cloud_pubsub::client::{Client, ClientConfig};
use google_cloud_pubsub::subscription::SubscriptionConfig;
use google_cloud_gax::conn::Environment;

fn get_free_port() -> u16 {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind to free port");
    socket.local_addr().expect("Failed to get local addr").port()
}

struct BackgroundProcess {
    child: Child,
    name: String,
}

impl BackgroundProcess {
    fn new(mut cmd: Command, name: &str) -> Self {
        let child = cmd.spawn().unwrap_or_else(|e| panic!("Failed to spawn {}: {}", name, e));
        Self {
            child,
            name: name.to_string(),
        }
    }
}

impl Drop for BackgroundProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        eprintln!("[E2E] Stopped {}", self.name);
    }
}

fn generate_base_config(output_dir: &str) -> String {
    format!(
        r##"[settings]
station = "R6E01"
output_dir = "{output_dir}"
debug = false

[printdata]
enabled = false

[write]
enabled = false
channels = ["all"]

[plot]
enabled = false
duration = 30
refresh_interval = 0
spectrogram = false
fullscreen = false
kiosk = false
eq_screenshots = false
channels = ["all"]
filter_waveform = false
filter_spectrogram = false
filter_highpass = 0.7
filter_lowpass = 2.0
filter_corners = 4
spectrogram_freq_range = false
upper_limit = 50.0
lower_limit = 0.0
logarithmic_y_axis = false
deconvolve = false
units = "CHAN"

[forward]
enabled = false
address = ["127.0.0.1"]
port = [0]
channels = ["all"]
fwd_data = false
fwd_alarms = false

[alert]
enabled = true
channel = "HZ"
sta = 6.0
lta = 30.0
duration = 0.0
threshold = 4.5
reset = 1.5
highpass = 0.1
lowpass = 5.0
deconvolve = false
units = "VEL"
on_plot = false
on_plot_end_line_color = "#D72638"
on_plot_start_line_color = "#4C8BF5"

[alertsound]
enabled = false
trigger_file = ""
default_reset_file = ""

[alertsound.intensity_files]

[custom]
enabled = false
codefile = "n/a"
win_override = false

[tweets]
enabled = false
tweet_images = false
api_key = "n/a"
api_secret = "n/a"
access_token = "n/a"
access_secret = "n/a"
extra_text = ""

[telegram]
enabled = false
send_images = false
token = "n/a"
chat_id = "n/a"
extra_text = ""
upload_timeout = 10

[googlechat]
enabled = false
webhook_url = "n/a"
extra_text = ""
send_images = false
s3_object_key_prefix = ""
s3_upload_timeout_seconds = 3

[discord]
enabled = false
webhook_url = "n/a"
use_embed = false
send_images = false
extra_text = ""

[sns]
enabled = false
topic_arn = "n/a"
aws_region = "n/a"
extra_text = ""

[line]
enabled = false
channel_access_token = "n/a"
to_ids = ""
extra_text = ""
send_images = false
s3_object_key_prefix = ""
s3_upload_timeout_seconds = 3

[bluesky]
enabled = false
post_images = false
username = "n/a"
password = "n/a"
extra_text = ""

[rsam]
enabled = false
quiet = true
fwaddr = "192.168.1.254"
fwport = 8887
fwformat = "LITE"
channel = "HZ"
interval = 10
deconvolve = false
units = "VEL"

[hue]
enabled = false
app_key = ""
target_ids = []
"##
    )
}

fn generate_publisher_config(
    udp_port: u16,
    _web_port: u16,
    output_dir: &str,
    topic: &str,
    project_id: &str,
) -> String {
    let mut config = generate_base_config(output_dir);
    config.push_str(&format!(
        r#"
[pubsub]
enabled = true
project_id = "{project_id}"
topic = "{topic}"
subscription = ""
input_mode = "udp"
batch_interval_ms = 500
"#
    ));
    // Insert port at the top of [settings]
    config = config.replacen(
        "[settings]\nstation",
        &format!("[settings]\nport = {udp_port}\nstation"),
        1,
    );
    config
}

fn generate_subscriber_config(
    _web_port: u16,
    output_dir: &str,
    subscription: &str,
    project_id: &str,
) -> String {
    let mut config = generate_base_config(output_dir);
    config.push_str(&format!(
        r#"
[pubsub]
enabled = true
project_id = "{project_id}"
topic = ""
subscription = "{subscription}"
input_mode = "pubsub"
batch_interval_ms = 500
"#
    ));
    // Insert port at the top of [settings]
    config = config.replacen(
        "[settings]\nstation",
        "[settings]\nport = 0\nstation",
        1,
    );
    config
}

#[test]
fn test_pubsub_e2e_roundtrip() {
    let emulator_host = match std::env::var("PUBSUB_EMULATOR_HOST") {
        Ok(val) if !val.is_empty() => val,
        _ => {
            eprintln!("Skipping E2E: PUBSUB_EMULATOR_HOST not set");
            return;
        }
    };

    let mseed_path = "../references/mseed/fdsnws.mseed";
    if !std::path::Path::new(mseed_path).exists() {
        eprintln!("Skipping E2E: MiniSEED test data not found at {}", mseed_path);
        return;
    }

    let project_id = "test-project";
    let topic = format!("e2e-topic-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
    let subscription = format!("e2e-sub-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());

    // Create topic and subscription in emulator
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let config = ClientConfig {
            environment: Environment::Emulator(emulator_host.clone()),
            project_id: Some(project_id.to_string()),
            ..Default::default()
        };
        let client = Client::new(config).await.expect("Failed to create emulator client");
        let t = client.topic(&topic);
        t.create(None, None).await.expect("Failed to create topic");
        let cfg = SubscriptionConfig {
            enable_message_ordering: true,
            ..Default::default()
        };
        client
            .create_subscription(&subscription, &topic, cfg, None)
            .await
            .expect("Failed to create subscription");
        eprintln!("[E2E] Created topic '{}' and subscription '{}'", topic, subscription);
    });

    let pub_udp_port = get_free_port();
    let pub_web_port = get_free_port();
    let sub_web_port = get_free_port();

    let pub_dir = tempdir().unwrap();
    let sub_dir = tempdir().unwrap();
    let pub_log = pub_dir.path().join("publisher.log");
    let sub_log = sub_dir.path().join("subscriber.log");

    // Write config files
    let pub_config_path = pub_dir.path().join("publisher.toml");
    let sub_config_path = sub_dir.path().join("subscriber.toml");

    fs::write(
        &pub_config_path,
        generate_publisher_config(
            pub_udp_port,
            pub_web_port,
            pub_dir.path().to_str().unwrap(),
            &topic,
            project_id,
        ),
    )
    .unwrap();

    fs::write(
        &sub_config_path,
        generate_subscriber_config(
            sub_web_port,
            sub_dir.path().to_str().unwrap(),
            &subscription,
            project_id,
        ),
    )
    .unwrap();

    eprintln!(
        "[E2E] Publisher: UDP={}, Web={}, Config={:?}",
        pub_udp_port, pub_web_port, pub_config_path
    );
    eprintln!(
        "[E2E] Subscriber: Web={}, Config={:?}",
        sub_web_port, sub_config_path
    );

    // Use pre-compiled binaries (avoid cargo lock contention)
    let rsudp_bin = env!("CARGO_BIN_EXE_rsudp-rust");
    let streamer_bin = env!("CARGO_BIN_EXE_streamer");

    // Spawn subscriber first (so it's ready when publisher starts sending)
    let sub_log_file = fs::File::create(&sub_log).unwrap();
    let mut sub_cmd = Command::new(rsudp_bin);
    sub_cmd
        .arg("--config")
        .arg(&sub_config_path)
        .arg("--web-port")
        .arg(sub_web_port.to_string())
        .arg("--station")
        .arg("R6E01")
        .arg("--network")
        .arg("AM")
        .env("PUBSUB_EMULATOR_HOST", &emulator_host)
        .env("RUST_LOG", "rsudp_rust=info")
        .stdout(sub_log_file.try_clone().unwrap())
        .stderr(sub_log_file);
    let _sub_proc = BackgroundProcess::new(sub_cmd, "subscriber");

    // Spawn publisher
    let pub_log_file = fs::File::create(&pub_log).unwrap();
    let mut pub_cmd = Command::new(rsudp_bin);
    pub_cmd
        .arg("--config")
        .arg(&pub_config_path)
        .arg("--udp-port")
        .arg(pub_udp_port.to_string())
        .arg("--web-port")
        .arg(pub_web_port.to_string())
        .arg("--station")
        .arg("R6E01")
        .arg("--network")
        .arg("AM")
        .env("PUBSUB_EMULATOR_HOST", &emulator_host)
        .env("RUST_LOG", "rsudp_rust=info")
        .stdout(pub_log_file.try_clone().unwrap())
        .stderr(pub_log_file);
    let _pub_proc = BackgroundProcess::new(pub_cmd, "publisher");

    // Wait for both processes to start up
    thread::sleep(Duration::from_secs(5));

    // Spawn streamer
    let mut streamer_cmd = Command::new(streamer_bin);
    streamer_cmd
        .arg("--file")
        .arg(mseed_path)
        .arg("--addr")
        .arg(format!("127.0.0.1:{}", pub_udp_port))
        .arg("--speed")
        .arg("100.0");
    let _streamer_proc = BackgroundProcess::new(streamer_cmd, "streamer");

    // Wait for data to flow through the pipeline
    let start = Instant::now();
    let timeout = Duration::from_secs(60);
    let mut pub_published = false;
    let mut sub_received = false;
    let mut sub_injected = false;

    while start.elapsed() < timeout {
        // Check publisher log for publish confirmation
        if !pub_published {
            if let Ok(mut f) = fs::File::open(&pub_log) {
                let mut contents = String::new();
                f.read_to_string(&mut contents).unwrap_or_default();
                if contents.contains("Published batch") {
                    eprintln!("[E2E] Publisher: batch published");
                    pub_published = true;
                }
            }
        }

        // Check subscriber log for received data
        if !sub_received {
            if let Ok(mut f) = fs::File::open(&sub_log) {
                let mut contents = String::new();
                f.read_to_string(&mut contents).unwrap_or_default();
                if contents.contains("Received batch") {
                    eprintln!("[E2E] Subscriber: batch received");
                    sub_received = true;
                }
                if contents.contains("Injected") {
                    eprintln!("[E2E] Subscriber: data injected into pipeline");
                    sub_injected = true;
                }
            }
        }

        if pub_published && sub_received && sub_injected {
            break;
        }

        thread::sleep(Duration::from_millis(500));
    }

    // Dump logs for debugging
    if !pub_published || !sub_received {
        eprintln!("--- PUBLISHER LOG ---");
        if let Ok(c) = fs::read_to_string(&pub_log) {
            eprintln!("{}", c);
        }
        eprintln!("--- SUBSCRIBER LOG ---");
        if let Ok(c) = fs::read_to_string(&sub_log) {
            eprintln!("{}", c);
        }
    }

    assert!(pub_published, "Publisher should have published at least one batch");
    assert!(sub_received, "Subscriber should have received at least one batch");
    assert!(
        sub_injected,
        "Subscriber should have injected channel data into pipeline"
    );

    eprintln!(
        "[E2E] Pub/Sub round-trip test PASSED in {}s",
        start.elapsed().as_secs()
    );
}
