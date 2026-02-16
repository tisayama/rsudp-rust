# Feature Specification: Google Cloud Pub/Sub Integration

**Feature Branch**: `040-pubsub-integration`
**Created**: 2026-02-13
**Status**: Draft
**Input**: User description: "Google Cloud Pub/Subでpublish, subscribe それぞれできる仕組みを作ってほしいです。オンプレミスで走らせるので、認証はservice account jsonで、環境変数か、パス指定でjson読めるようにしてください。パブリッシャーは複数インスタンスで動く可能性があるため、ライブデータのUDPパケットのタイムスタンプから、予測可能なキーIDを生成し、一度しか処理されないことを担保してください。Pub/Subのベストプラクティスでそういったものがあれば従ってください。また、今回は感度係数の調整前の数値のまま配信してほしいですが、チャンネルごとに10Hzのパケットは多すぎるので、全チャンネル集約し、0.5秒ごとにバッチ送信してほしいです。また、テストについてはPub/Subシミュレーターがあればモックと併用しそれを使ってほしいです。なければモックだけでよいです。"

## Clarifications

### Session 2026-02-13

- Q: What serialization format should be used for Pub/Sub message payloads? → A: Protocol Buffers (compact binary with schema sharing)
- Q: How should the subscriber be integrated? → A: As an alternative input source to the rsudp-rust pipeline, replacing UDP stream reception. The subscriber feeds received data into the same processing pipeline (trigger, WebUI, alerts, etc.).
- Q: Should message ordering be guaranteed for time-series correctness? → A: Yes, use Pub/Sub ordering keys (per station) to guarantee in-order delivery.
- Q: Should E2E tests be included? → A: Yes. E2E test validates full round-trip: UDP streamer → publisher → Pub/Sub emulator → subscriber → pipeline, verifying data integrity and deduplication.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Publish Seismic Data to Pub/Sub (Priority: P1)

An operator runs rsudp-rust on-premises with multiple instances receiving UDP seismic data from Raspberry Shake sensors. The system publishes raw (pre-deconvolution) sample data to a Google Cloud Pub/Sub topic. All channels (EHZ, ENE, ENN, ENZ) received within a 0.5-second window are aggregated into a single message and published as a batch. Even when multiple rsudp-rust instances receive the same UDP packets, each data point is published exactly once thanks to deterministic message deduplication keys derived from packet timestamps.

**Why this priority**: Publishing is the core capability -- without it, no downstream consumer can receive data.

**Independent Test**: Can be tested by starting rsudp-rust with Pub/Sub enabled, streaming sample data via the streamer tool, and verifying messages appear in the topic (or emulator).

**Acceptance Scenarios**:

1. **Given** rsudp-rust is configured with a valid service account JSON and Pub/Sub topic, **When** UDP seismic packets arrive, **Then** all channels are aggregated and published as a single message every 0.5 seconds.
2. **Given** two rsudp-rust instances receive the same UDP packet, **When** both instances publish to the same topic, **Then** the subscriber receives the data only once (deduplicated by the deterministic message attribute).
3. **Given** rsudp-rust is configured with Pub/Sub enabled, **When** the published message is inspected, **Then** it contains raw sample values (integer counts before sensitivity/deconvolution adjustment) with station, channel, timestamp, and sample rate metadata.

---

### User Story 2 - Subscribe as Alternative Pipeline Input (Priority: P2)

An operator runs a second rsudp-rust instance at a remote site without direct UDP access to the Raspberry Shake sensor. Instead of receiving UDP packets, this instance subscribes to the Pub/Sub topic and uses the received seismic data as its pipeline input. The full processing pipeline (STA/LTA trigger, WebUI, alerts, intensity calculation, etc.) operates identically regardless of whether the input comes from UDP or Pub/Sub.

**Why this priority**: The subscriber enables remote monitoring without direct network access to the sensor, but publishing must work first.

**Independent Test**: Can be tested by publishing known test data to a topic (or emulator), starting rsudp-rust in Pub/Sub subscriber mode, and verifying the WebUI displays waveforms and the trigger system fires alerts.

**Acceptance Scenarios**:

1. **Given** rsudp-rust is configured with a Pub/Sub subscription as its input source, **When** data is published to the topic, **Then** the pipeline processes it identically to UDP-received data (triggers, WebUI, alerts all function).
2. **Given** Pub/Sub delivers the same message twice (at-least-once delivery), **When** the subscriber processes both deliveries, **Then** the data is processed only once (application-level deduplication using the message attribute).
3. **Given** a subscriber-mode rsudp-rust is running, **When** the publisher is temporarily unavailable and then resumes, **Then** the subscriber catches up on any backlog without data loss.
4. **Given** rsudp-rust is configured in subscriber mode, **When** it starts, **Then** it does not listen for UDP packets (Pub/Sub replaces UDP as the data source).

---

### User Story 3 - Service Account Authentication (Priority: P1)

An operator configures rsudp-rust to authenticate with Google Cloud Pub/Sub using a service account JSON key file. The key file path can be specified either via the `GOOGLE_APPLICATION_CREDENTIALS` environment variable or via a configuration field in `rsudp.toml`. The system runs on-premises (not on GCP infrastructure), so cloud metadata-based auth is not available.

**Why this priority**: Authentication is required for any Pub/Sub operation -- it is a prerequisite for both publishing and subscribing.

**Independent Test**: Can be tested by providing a service account JSON and verifying the system connects to Pub/Sub (or emulator) without authentication errors.

**Acceptance Scenarios**:

1. **Given** the `GOOGLE_APPLICATION_CREDENTIALS` environment variable points to a valid service account JSON file, **When** rsudp-rust starts with Pub/Sub enabled, **Then** it authenticates successfully and publishes data.
2. **Given** a `credentials_file` path is set in the `[pubsub]` section of `rsudp.toml`, **When** rsudp-rust starts, **Then** it uses that file for authentication.
3. **Given** neither environment variable nor config path is set, **When** rsudp-rust starts with Pub/Sub enabled, **Then** it logs a clear error message indicating that credentials are required and disables Pub/Sub functionality without crashing.

---

### User Story 4 - Emulator-Based Testing (Priority: P2)

A developer runs integration tests locally using the Google Cloud Pub/Sub emulator. When the `PUBSUB_EMULATOR_HOST` environment variable is set, the system connects to the emulator instead of the real Pub/Sub service, requiring no credentials. Tests cover the full publish-subscribe flow including batching, deduplication, and message format.

**Why this priority**: Testing infrastructure is essential for quality but is not end-user-facing functionality.

**Independent Test**: Can be tested by starting the Pub/Sub emulator in Docker, setting the emulator host environment variable, and running the integration test suite.

**Acceptance Scenarios**:

1. **Given** the `PUBSUB_EMULATOR_HOST` environment variable is set, **When** rsudp-rust starts with Pub/Sub enabled, **Then** it connects to the emulator and publishes/subscribes without real GCP credentials.
2. **Given** the emulator is running, **When** the integration test suite executes, **Then** all publish and subscribe tests pass using the emulator.

---

### User Story 5 - E2E Test: Full Pipeline via Pub/Sub (Priority: P2)

A developer runs an end-to-end test that validates the complete data flow: streamer sends UDP packets to a publisher instance, the publisher aggregates and publishes to the Pub/Sub emulator, and a subscriber instance receives from the emulator and feeds data into its processing pipeline. The test verifies that seismic data arrives intact through the full Pub/Sub round-trip, including correct sample values, timestamps, channel metadata, and deduplication.

**Why this priority**: E2E tests provide the highest confidence that all components work together correctly, but depend on both publisher and subscriber being functional first.

**Independent Test**: Can be tested by running the Pub/Sub emulator in Docker, starting two rsudp-rust processes (one as publisher with UDP input, one as subscriber with Pub/Sub input), streaming MiniSEED data via the streamer, and verifying data integrity at the subscriber end.

**Acceptance Scenarios**:

1. **Given** the Pub/Sub emulator is running and two rsudp-rust instances are configured (publisher: UDP input + Pub/Sub output, subscriber: Pub/Sub input), **When** the streamer sends MiniSEED data via UDP, **Then** the subscriber instance receives all data with correct raw sample values and timestamps.
2. **Given** the E2E test environment, **When** the streamer sends 30 seconds of MiniSEED data, **Then** no data windows are lost or duplicated (verified by comparing deduplication keys at publisher and subscriber).
3. **Given** the E2E test environment, **When** the subscriber receives data, **Then** the data can be processed by the pipeline (waveform buffers are populated, trigger system can operate).

---

### Edge Cases

- What happens when the Pub/Sub service is temporarily unreachable? The publisher retries with exponential backoff and does not lose buffered data (up to a configurable memory limit).
- What happens when a message exceeds Pub/Sub's 10 MB size limit? This should never occur for 0.5-second batches of seismic data (typical message ~1-5 KB), but the system validates message size before publishing and logs a warning if it approaches the limit.
- What happens when the 0.5-second aggregation window has no data for some channels? The batch includes only channels that received data during that window; empty channels are omitted from the message.
- What happens when clock skew causes duplicate timestamps across different instances? The deduplication key includes the station identifier and all channel names, making it unique per data window regardless of which instance publishes it.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST publish seismic sample data to a configurable Google Cloud Pub/Sub topic.
- **FR-002**: System MUST support a Pub/Sub subscriber mode that receives seismic data from a subscription and feeds it into the existing processing pipeline (trigger, WebUI, alerts) as an alternative to UDP input.
- **FR-003**: System MUST authenticate using a service account JSON key file, discoverable via the `GOOGLE_APPLICATION_CREDENTIALS` environment variable or a `credentials_file` path in configuration.
- **FR-004**: System MUST aggregate all channels' data within each 0.5-second window into a single Pub/Sub message, rather than publishing per-channel per-packet.
- **FR-005**: System MUST publish raw sample values (integer counts before sensitivity coefficient adjustment or deconvolution).
- **FR-006**: System MUST generate a deterministic deduplication key from the data's timestamp window and station identifier, attached as a message attribute, so that duplicate publishes from multiple instances can be detected.
- **FR-007**: Subscribers MUST deduplicate received messages using the deduplication key attribute to guarantee each data window is processed only once.
- **FR-008**: System MUST connect to the Pub/Sub emulator when the `PUBSUB_EMULATOR_HOST` environment variable is set, bypassing real credentials.
- **FR-009**: System MUST be configurable via a `[pubsub]` section in `rsudp.toml` with fields for: `enabled`, `project_id`, `topic`, `subscription` (for subscriber mode), and `credentials_file` (optional).
- **FR-010**: System MUST include station name, channel identifiers, sample rate, and timestamp metadata in each published message.
- **FR-011**: System MUST retry failed publishes with exponential backoff without blocking the main data pipeline.
- **FR-012**: System MUST serialize message payloads using Protocol Buffers with a shared `.proto` schema definition for cross-language subscriber compatibility.
- **FR-013**: System MUST use Pub/Sub ordering keys (keyed by station identifier) to guarantee messages are delivered to subscribers in publish order, preserving time-series continuity.
- **FR-014**: An E2E test MUST validate the full round-trip: UDP streamer → publisher instance → Pub/Sub emulator → subscriber instance → pipeline processing, verifying data integrity (sample values, timestamps, channel metadata) and deduplication correctness.

### Key Entities

- **Pub/Sub Message**: A batched seismic data payload covering a 0.5-second window, serialized as Protocol Buffers. Contains raw integer samples for all active channels, along with metadata (station, channel list, sample rate, window start/end timestamps). Carries a `dedup_key` attribute for idempotent processing. A `.proto` schema definition is shared with subscribers for deserialization. Published with an ordering key (station identifier) to guarantee sequential delivery.
- **Deduplication Key**: A deterministic string derived from the station identifier and the 0.5-second window's start timestamp (e.g., `AM.R6E01:2025-11-25T09:01:23.500Z`). Identical across all publisher instances for the same data window.
- **Pub/Sub Configuration**: Settings in `rsudp.toml` governing Pub/Sub behavior: project ID, topic name, subscription name, credentials path, and enable/disable toggle.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All active channels' data within a 0.5-second window is published as a single message, reducing message volume by at least 80% compared to per-channel per-packet publishing.
- **SC-002**: When two publisher instances process the same UDP data simultaneously, a subscriber receives each 0.5-second data window exactly once (zero duplicates after application-level deduplication).
- **SC-003**: Publishing latency from data arrival to Pub/Sub acknowledgment is under 2 seconds in normal operation (excluding network latency to GCP).
- **SC-004**: The system gracefully handles Pub/Sub service interruptions, buffering data and resuming publishing within 30 seconds of connectivity restoration without data loss.
- **SC-005**: Integration tests using the Pub/Sub emulator cover the full publish-subscribe-deduplicate cycle and pass reliably in CI environments.
- **SC-007**: An E2E test validates the complete round-trip (UDP → publisher → Pub/Sub emulator → subscriber → pipeline) with zero data loss or duplication over a 30-second test window.
- **SC-006**: Configuration can be completed in under 5 minutes by an operator with a service account JSON file and a project ID.

## Assumptions

- The Pub/Sub emulator (available as a Docker image: `gcr.io/google.com/cloudsdktool/google-cloud-cli:emulators`) will be used for integration testing. Mock-based unit tests supplement emulator tests for faster CI feedback.
- Pub/Sub's at-least-once delivery model is accepted; application-level deduplication on the subscriber side provides the exactly-once processing guarantee.
- The deduplication key is based on a floored 0.5-second timestamp window (e.g., timestamp 09:01:23.730 maps to window start 09:01:23.500). This means all publishers processing data from the same time window generate the same key.
- Raw sample values are signed 32-bit integers representing digitizer counts. No unit conversion is applied before publishing.
- The subscriber operates as an alternative input source to the rsudp-rust pipeline, replacing UDP reception. When in subscriber mode, UDP listening is disabled and all data flows from Pub/Sub into the same pipeline stages.
- The `google-cloud-pubsub` Rust crate (official Google-maintained client) will be used. It is pre-1.0 but actively developed; the alternative `gcloud-sdk` crate is the fallback.
