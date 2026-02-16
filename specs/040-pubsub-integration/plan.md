# Implementation Plan: Google Cloud Pub/Sub Integration

**Branch**: `040-pubsub-integration` | **Date**: 2026-02-13 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/040-pubsub-integration/spec.md`

## Summary

Add Google Cloud Pub/Sub publish/subscribe functionality to rsudp-rust. The publisher aggregates seismic data received via UDP into 0.5-second windows across all channels and publishes it to a Pub/Sub topic in Protocol Buffers format. The subscriber uses a Pub/Sub subscription as an alternative to UDP input, feeding data into the existing pipeline (triggers, WebUI, alerts, etc.) in the same format. Authentication uses service account JSON keys. Testing uses the Pub/Sub emulator.

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2021)
**Primary Dependencies**: `google-cloud-pubsub` (v0.30+, Pub/Sub client), `prost` (v0.14, Protocol Buffers), `prost-build` (v0.14, build-time code generation), `tokio` (existing, async runtime)
**Storage**: N/A (in-memory buffering only)
**Testing**: `cargo test` + Pub/Sub emulator (Docker) + mock-based unit tests
**Target Platform**: Linux (on-premises)
**Project Type**: Single project (feature addition to existing rsudp-rust)
**Performance Goals**: Publish latency < 2s (data arrival to Pub/Sub ack), no main pipeline blocking
**Constraints**: Pub/Sub message size < 10MB (actual ~1.2KB, well within limits), configurable memory buffer cap
**Scale/Scope**: 1 station × 4 channels × 100Hz, multi-publisher instance support

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Stability & Reliability | PASS | Retry + buffering on Pub/Sub disconnection. Deduplication guarantees correctness. Pipeline unaffected by Pub/Sub failures |
| II. Rigorous Testing | PASS | 3-tier approach: unit (mock) + integration (emulator) + E2E |
| III. High Performance | PASS | Async publish (tokio::spawn). 0.5s batching reduces message volume by 80%+. Main pipeline never blocked |
| IV. Code Clarity | PASS | Module structure follows existing patterns (SNSManager, ForwardManager) |
| V. Japanese Specification | PASS | Spec written in Japanese; plan and docs in English per user request |
| VI. Standard Tech Stack | N/A | No WebUI changes (backend only) |
| VII. Self-Verification | PASS | Emulator-based integration tests for verification |
| VIII. Branch Strategy | PASS | Working on `040-pubsub-integration` branch |

## Project Structure

### Documentation (this feature)

```text
specs/040-pubsub-integration/
├── plan.md              # This file
├── research.md          # Phase 0: Technology research findings
├── data-model.md        # Phase 1: Entity definitions
├── quickstart.md        # Phase 1: Developer quickstart guide
├── contracts/
│   ├── seismic.proto    # Phase 1: Protocol Buffers schema
│   └── pubsub-config.toml  # Phase 1: Config file example
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2: Task breakdown (/speckit.tasks)
```

### Source Code (repository root)

```text
rsudp-rust/
├── proto/
│   └── seismic.proto           # Protobuf schema definition
├── build.rs                    # prost-build code generation (new)
├── src/
│   ├── lib.rs                  # Add pub mod pubsub;
│   ├── settings.rs             # Add PubsubSettings
│   ├── main.rs                 # Pub/Sub init + input mode switching
│   ├── pipeline.rs             # Data feed to publisher
│   └── pubsub/
│       ├── mod.rs              # Module exports
│       ├── publisher.rs        # PubsubPublisher (buffer + batch send)
│       ├── subscriber.rs       # PubsubSubscriber (receive + pipeline injection)
│       ├── dedup.rs            # Deduplication key generation + checking
│       └── proto.rs            # Generated code re-export
├── tests/
│   ├── pubsub_unit.rs          # Mock-based unit tests
│   ├── pubsub_integration.rs   # Emulator-based integration tests
│   └── pubsub_e2e.rs           # Full pipeline E2E tests
└── Cargo.toml                  # Add dependencies
```

**Structure Decision**: Add a `pubsub/` submodule to the existing rsudp-rust single-project structure. Follows the same pattern as SNSManager and ForwardManager — initialized from `main.rs` and integrated with `pipeline.rs`.

## Phase 0 Artifacts

- [research.md](./research.md): All technology research complete. No NEEDS CLARIFICATION remaining.

## Phase 1 Artifacts

- [data-model.md](./data-model.md): SeismicBatch, ChannelData, PubsubSettings entity definitions
- [contracts/seismic.proto](./contracts/seismic.proto): Protocol Buffers schema
- [contracts/pubsub-config.toml](./contracts/pubsub-config.toml): Config file example
- [quickstart.md](./quickstart.md): Developer quickstart guide

## Implementation Approach

### Module Design

#### 1. `pubsub/publisher.rs` - PubsubPublisher

```rust
pub struct PubsubPublisher {
    publisher: Publisher,          // google-cloud-pubsub Publisher
    buffer: HashMap<String, Vec<ChannelData>>,  // Channel buffer
    station: String,
    sample_rate: f64,
    batch_interval: Duration,
    window_start: Option<i64>,
}

impl PubsubPublisher {
    pub async fn new(config: &PubsubSettings, station: &str) -> Result<Self>;
    pub fn buffer_segment(&mut self, channel: &str, samples: &[i32], start_time_ms: i64, sample_rate: f64);
    pub async fn flush(&mut self) -> Result<()>;  // Batch send
}
```

- `buffer_segment()` is called from the main loop in `pipeline.rs`
- `flush()` is invoked from a 0.5-second interval tokio task
- Connected to the main pipeline asynchronously via `mpsc::Sender<SegmentData>`

#### 2. `pubsub/subscriber.rs` - PubsubSubscriber

```rust
pub struct PubsubSubscriber {
    subscription: Subscription,
    dedup: DedupChecker,
    pipe_tx: mpsc::Sender<Vec<u8>>,
}

impl PubsubSubscriber {
    pub async fn new(config: &PubsubSettings, pipe_tx: mpsc::Sender<Vec<u8>>) -> Result<Self>;
    pub async fn run(&self, cancel: CancellationToken) -> Result<()>;
}
```

- `run()` processes messages in the `subscription.receive()` callback
- Decodes protobuf data and converts to rsudp JSON packet format
- Injects into the existing pipeline via `pipe_tx.send()`

#### 3. `pubsub/dedup.rs` - Deduplication

```rust
pub fn generate_dedup_key(station: &str, timestamp_ms: i64) -> String;
pub struct DedupChecker {
    seen: HashSet<String>,
    order: VecDeque<String>,
    max_entries: usize,
}
```

- `generate_dedup_key()`: Floors timestamp to 500ms boundary to generate the key
- `DedupChecker`: LRU-style HashSet for duplicate detection

#### 4. `main.rs` Integration

```rust
// Branch on input_mode
if settings.pubsub.enabled && settings.pubsub.input_mode == "pubsub" {
    // Subscriber mode: no UDP listener
    let subscriber = PubsubSubscriber::new(&settings.pubsub, pipe_tx).await?;
    tokio::spawn(async move { subscriber.run(cancel).await; });
} else {
    // UDP mode (default)
    let (recv_tx, mut recv_rx) = mpsc::channel(100);
    tokio::spawn(start_receiver(udp_port, recv_tx));
    // ... existing recv_rx → pipe_tx bridge

    // Publisher (optional)
    if settings.pubsub.enabled {
        let publisher = PubsubPublisher::new(&settings.pubsub, &sta).await?;
        // Pass publisher to pipeline
    }
}
```

## Complexity Tracking

> No constitution check violations. No additional justification required.
