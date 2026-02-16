# Quickstart: Google Cloud Pub/Sub Integration

## Prerequisites

- rsudp-rust build/run environment ready
- Google Cloud project (with a service account JSON key)
- Docker (for emulator testing)

## 1. Publisher Mode

### 1.1 Configuration (`rsudp.toml`)

```toml
[settings]
port = 8888
station = "R6E01"

[pubsub]
enabled = true
project_id = "my-gcp-project"
topic = "seismic-data"
input_mode = "udp"
credentials_file = "/path/to/service-account.json"
```

### 1.2 Start

```bash
# Option 1: Specify credentials_file in config
cargo run -- -C rsudp.toml

# Option 2: Specify via environment variable
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
cargo run -- -C rsudp.toml
```

rsudp-rust receives UDP packets and publishes batched data to the Pub/Sub topic every 0.5 seconds.

## 2. Subscriber Mode

### 2.1 Configuration

```toml
[settings]
station = "R6E01"

[pubsub]
enabled = true
project_id = "my-gcp-project"
subscription = "seismic-data-sub"
input_mode = "pubsub"
credentials_file = "/path/to/service-account.json"
```

### 2.2 Start

```bash
cargo run -- -C rsudp.toml
```

The UDP listener is not started. Data is received from the Pub/Sub subscription and fed into the pipeline. WebUI, triggers, alerts, etc. all function normally.

## 3. Local Testing with Emulator

### 3.1 Start Emulator

```bash
docker run -d --name pubsub-emulator \
  -p 8085:8085 \
  gcr.io/google.com/cloudsdktool/google-cloud-cli:emulators \
  gcloud beta emulators pubsub start --host-port=0.0.0.0:8085
```

### 3.2 Run Tests

```bash
export PUBSUB_EMULATOR_HOST=localhost:8085
cargo test --test pubsub_integration
```

### 3.3 E2E Test

```bash
# Start all components with Docker Compose
docker compose -f docker-compose.test.yml up -d

# Run E2E test
cargo test --test pubsub_e2e
```

## 4. Multi-Instance Operation

Even when multiple rsudp-rust instances receive the same UDP data, the deterministic deduplication key (station name + timestamp window) ensures subscribers process data exactly once.

```
rsudp-rust (Instance 1) ──→ Pub/Sub Topic ──→ Subscription ──→ rsudp-rust (Subscriber)
rsudp-rust (Instance 2) ──↗   (same dedup_key → processed once)
```

## 5. Verification

Publisher logs should show:
```
INFO pubsub: Publishing batch: station=AM.R6E01, channels=4, samples=200, window=2025-11-25T09:01:23.500Z
INFO pubsub: Published successfully, dedup_key=AM.R6E01:2025-11-25T09:01:23.500Z
```

Subscriber logs:
```
INFO pubsub: Received batch: station=AM.R6E01, channels=4, dedup_key=AM.R6E01:2025-11-25T09:01:23.500Z
INFO pubsub: Injected 4 channel segments into pipeline
```
