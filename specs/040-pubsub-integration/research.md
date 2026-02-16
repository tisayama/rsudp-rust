# Research: Google Cloud Pub/Sub Integration

**Feature**: 040-pubsub-integration | **Date**: 2026-02-13

## 1. Pub/Sub Rust Client Library

### Decision: `google-cloud-pubsub` crate (v0.30+)

**Rationale**: Google-maintained official Rust client. Natively supports the tokio async runtime and communicates with the Pub/Sub API via gRPC (tonic). Automatically detects the `PUBSUB_EMULATOR_HOST` environment variable for emulator connectivity.

**Alternatives considered**:
- `gcloud-sdk`: Broader GCP service coverage, but Pub/Sub-specific API maturity is inferior to `google-cloud-pubsub`
- Direct gRPC/REST: Maintenance cost too high

**API patterns**:
```rust
// Service account authentication
let cred = CredentialsFile::new_from_file(path).await?;
let config = ClientConfig::default().with_credentials(cred).await?;
let client = Client::new(config).await?;

// Emulator connection (auto-detected when PUBSUB_EMULATOR_HOST is set, no auth required)
let config = ClientConfig::default();
let client = Client::new(config).await?;

// Topic/Subscription retrieval
let topic = client.topic("my-topic");
let subscription = client.subscription("my-sub");

// Publisher (with ordering key + attributes)
let publisher = topic.new_publisher(None);
let msg = PubsubMessage {
    data: payload.to_vec(),
    attributes: HashMap::from([("dedup_key".into(), key.into())]),
    ordering_key: station_id.to_string(),
    ..Default::default()
};
let awaiter = publisher.publish(msg).await;
awaiter.get().await?; // Wait for acknowledgment

// Subscriber
let cancel = CancellationToken::new();
subscription.receive(|mut message, _cancel| async move {
    let data = &message.message.data;
    let attrs = &message.message.attributes;
    // Process...
    message.ack().await.ok();
}, cancel, None).await?;
```

**Cargo dependencies**:
```toml
google-cloud-pubsub = "0.30"
google-cloud-googleapis = "0.16"
google-cloud-gax = "0.19"
```

## 2. Protocol Buffers Serialization

### Decision: `prost` crate (v0.14) + `prost-build` (build-time code generation)

**Rationale**: The most widely used protobuf implementation in the Rust ecosystem. Automatically generates Rust structs from `.proto` files via `build.rs`, providing type-safe serialization/deserialization.

**Alternatives considered**:
- `protobuf` (stepancheg): Viable but `prost` has broader ecosystem adoption
- `serde_json`: JSON is human-readable but ~5-10x less size-efficient than binary protobuf
- MessagePack: Cannot share schemas via `.proto` files

**Implementation pattern**:
```rust
// build.rs
fn main() {
    prost_build::compile_protos(
        &["proto/seismic.proto"],
        &["proto/"],
    ).unwrap();
}

// Using generated code
include!(concat!(env!("OUT_DIR"), "/rsudp.seismic.rs"));

// Serialize / Deserialize
let bytes = msg.encode_to_vec();
let decoded = SeismicBatch::decode(&bytes[..])?;
```

**Cargo dependencies**:
```toml
prost = "0.14"
prost-types = "0.14"

[build-dependencies]
prost-build = "0.14"
```

## 3. Pub/Sub Best Practices

### 3.1 Deduplication

Pub/Sub does not have native deduplication (at-least-once delivery). Application-level implementation is required.

**Decision**: Attach a deterministic key to the `dedup_key` message attribute; subscriber uses a HashSet-based duplicate detector.

**Key generation rule**: `{station}:{window_start_iso8601}`
- Example: `AM.R6E01:2025-11-25T09:01:23.500Z`
- Floor the 0.5-second window start timestamp to 500ms boundary
- Any publisher instance processing data from the same window generates the identical key

**Subscriber-side implementation**:
- Maintain a `HashSet<String>` of the most recent N entries (e.g., 10,000 dedup_keys)
- If `dedup_key` exists in the set on receipt, immediately ack (skip processing)
- LRU or time-based expiry to automatically purge old keys

### 3.2 Ordering Keys

**Decision**: Use station identifier as the ordering key to guarantee time-series ordering.

- Pub/Sub's ordering key feature delivers messages with the same key in publish order
- Key: `{network}.{station}` (e.g., `AM.R6E01`)
- Cross-station ordering is not required (each station is an independent time series)

### 3.3 Batch Processing

- Aggregate all channels (EHZ, ENE, ENN, ENZ, etc.) within a 0.5-second window into a single message
- Raspberry Shake 4D @ 100Hz → 0.5s = 50 samples/ch × 4 ch = 200 samples
- Estimated message size: ~1.2 KB (protobuf)
- Well within Pub/Sub's 10 MB limit

### 3.4 Error Handling and Retry

- The `google-cloud-pubsub` client implements gRPC-level retry internally
- Application layer adds exponential backoff retry on top
- Temporary disconnection: Accumulate data in memory buffer (configurable upper limit)
- Persistent failure: Log warning, main pipeline continues operating

## 4. Emulator-Based Testing

### Decision: `gcr.io/google.com/cloudsdktool/google-cloud-cli:emulators` Docker image

**Rationale**: Official Google emulator. Starts via `gcloud beta emulators pubsub start`. Used for local integration and E2E tests.

**Setup**:
```bash
# Docker startup
docker run -p 8085:8085 gcr.io/google.com/cloudsdktool/google-cloud-cli:emulators \
  gcloud beta emulators pubsub start --host-port=0.0.0.0:8085

# Environment variable
export PUBSUB_EMULATOR_HOST=localhost:8085
```

**Test strategy**:
1. **Unit tests (mock)**: Abstract publisher/subscriber logic behind traits, test with mocks
2. **Integration tests (emulator)**: Start emulator via Docker Compose, test with real Pub/Sub API
3. **E2E tests (emulator)**: Full path — UDP streamer → publisher → emulator → subscriber → pipeline

## 5. Codebase Architecture Analysis

### 5.1 Existing Pipeline Structure

```
UDP Socket → start_receiver() → mpsc::Sender<Packet> → recv_rx
    ↓ (main.rs L349-353)
packet.data → mpsc::Sender<Vec<u8>> pipe_tx → pipe_rx
    ↓
run_pipeline(pipe_rx, ...) → parse_any() → segments
    ↓
├── TriggerManager → AlertEvents → SNS/Hue/Audio/Email
├── IntensityManager → broadcast_intensity
├── ForwardManager → forward_data/forward_alarm
├── RsamManager → process_segment
└── WebState → broadcast_waveform → WebSocket clients
```

### 5.2 Publisher Integration Point

The publisher connects **in parallel** to the existing pipeline:
- Access raw sample data within `run_pipeline()`'s `for segment in segments` loop
- Feed data to the publisher at the same level as `web_state.broadcast_waveform()`
- 0.5-second buffering is handled internally by the publisher (`tokio::time::interval`)

### 5.3 Subscriber Integration Point

The subscriber functions as a **replacement** for the UDP receiver:
- Replaces `main.rs` L340-353 `start_receiver()` + recv_rx → pipe_tx bridge
- Decodes received protobuf messages and converts to rsudp JSON packet format
- Sends converted data via `pipe_tx.send(data)` into the existing pipeline
- UDP listener is not started (mutually exclusive via configuration)

### 5.4 Configuration Integration

Add `PubsubSettings` section to the existing `Settings` struct:
- Follow `SnsSettings` pattern with `enabled`, `project_id`, `topic`, `subscription`, `credentials_file`
- Add `"pubsub"` to `known_sections` (L616)
- `input_mode` field for mutually exclusive UDP/Pub/Sub input switching

### 5.5 Docker Integration

Add Pub/Sub emulator service to existing `docker-compose.yml`:
- `pubsub-emulator` service definition
- Inject `PUBSUB_EMULATOR_HOST` environment variable into the rsudp-rust service

## 6. Message Format Design

### Protobuf Schema Overview

```protobuf
message SeismicBatch {
  string station = 1;           // e.g., "AM.R6E01"
  int64 window_start_ms = 2;    // Window start time (Unix ms)
  int64 window_end_ms = 3;      // Window end time (Unix ms)
  double sample_rate = 4;       // Sample rate (Hz)
  repeated ChannelData channels = 5;
}

message ChannelData {
  string channel = 1;           // e.g., "EHZ"
  repeated int32 samples = 2;   // raw integer counts
  int64 start_time_ms = 3;      // Channel data start time
}
```

- Raw integer counts: Pre-sensitivity/deconvolution values (i32)
- Timestamps: Unix milliseconds (chrono-compatible)
- Station: network.station format
