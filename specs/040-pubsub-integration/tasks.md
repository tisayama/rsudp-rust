# Tasks: Google Cloud Pub/Sub Integration

**Input**: Design documents from `/specs/040-pubsub-integration/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are explicitly requested in the specification (emulator-based integration tests, E2E tests, and mock-based unit tests).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `rsudp-rust/src/`, `rsudp-rust/tests/` within the repository
- Proto files: `rsudp-rust/proto/`
- Build script: `rsudp-rust/build.rs`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add dependencies, protobuf schema, build-time code generation, and module skeleton

- [x] T001 Add `google-cloud-pubsub`, `prost`, `prost-types`, `tokio-util` (for CancellationToken) dependencies to `rsudp-rust/Cargo.toml` and `prost-build` to `[build-dependencies]`
- [x] T002 [P] Copy protobuf schema from `specs/040-pubsub-integration/contracts/seismic.proto` to `rsudp-rust/proto/seismic.proto`
- [x] T003 [P] Create `rsudp-rust/build.rs` with `prost_build::compile_protos()` to generate Rust code from `proto/seismic.proto`
- [x] T004 Create module skeleton: `rsudp-rust/src/pubsub/mod.rs` (empty pub mod declarations for publisher, subscriber, dedup, proto) and register `pub mod pubsub;` in `rsudp-rust/src/lib.rs`
- [x] T005 Verify `cargo build` compiles successfully with new dependencies and proto code generation

**Checkpoint**: Project compiles with new dependencies. Generated protobuf types are available via `crate::pubsub::proto`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**Warning**: No user story work can begin until this phase is complete

- [x] T006 Add `PubsubSettings` struct to `rsudp-rust/src/settings.rs` with fields: `enabled` (bool), `project_id` (String), `topic` (String), `subscription` (String), `credentials_file` (Option\<String\>), `input_mode` (String, default "udp"), `batch_interval_ms` (u64, default 500). Add `pubsub` field to `Settings` struct. Add `"pubsub"` to `known_sections` array. Implement `Default` for `PubsubSettings`.
- [x] T007 [P] Create `rsudp-rust/src/pubsub/proto.rs` that re-exports the prost-generated `SeismicBatch` and `ChannelData` types via `include!(concat!(env!("OUT_DIR"), "/rsudp.seismic.rs"))`
- [x] T008 [P] Implement `rsudp-rust/src/pubsub/dedup.rs`: `generate_dedup_key(station: &str, timestamp_ms: i64) -> String` that floors timestamp to 500ms boundary and formats as `{station}:{iso8601}`. Implement `DedupChecker` struct with `HashSet<String>` + `VecDeque<String>` for LRU eviction (max 10,000 entries), with methods `check_and_insert(&mut self, key: &str) -> bool` (returns true if new) and `len(&self) -> usize`.
- [x] T009 Update `rsudp-rust/src/pubsub/mod.rs` to export `pub mod proto; pub mod dedup; pub mod publisher; pub mod subscriber;`
- [x] T010 Verify `cargo build` compiles and `cargo test` passes with existing tests unaffected

**Checkpoint**: Foundation ready — PubsubSettings parsed from config, dedup logic available, proto types generated.

---

## Phase 3: User Story 3 - Service Account Authentication (Priority: P1) 🎯

**Goal**: Pub/Sub client initialization with service account JSON credentials, environment variable, or emulator auto-detection

**Independent Test**: Provide a service account JSON (or start the emulator), start rsudp-rust with `[pubsub] enabled = true`, and verify connection succeeds without auth errors

### Implementation for User Story 3

- [x] T011 [US3] Implement async `create_pubsub_client(config: &PubsubSettings) -> Result<Client>` function in `rsudp-rust/src/pubsub/mod.rs` that: (1) checks `PUBSUB_EMULATOR_HOST` env var first (use `ClientConfig::default()`, no auth), (2) checks `config.credentials_file` for path to JSON, (3) falls back to `GOOGLE_APPLICATION_CREDENTIALS` env var, (4) if none set and emulator not detected, returns error with clear message
- [x] T012 [US3] Add Pub/Sub client initialization to `rsudp-rust/src/main.rs`: after settings load and before pipeline start, call `create_pubsub_client()` when `settings.pubsub.enabled == true`. Log success or graceful error (disable pubsub, continue running). Store `Option<Client>` for use by publisher/subscriber.
- [x] T013 [US3] Add input_mode branching skeleton in `rsudp-rust/src/main.rs`: when `settings.pubsub.input_mode == "pubsub"`, skip UDP receiver startup; when `"udp"` (default), use existing UDP path. Both paths connect to `pipe_tx`.

### Tests for User Story 3

- [x] T014 [P] [US3] Unit test in `rsudp-rust/src/pubsub/mod.rs` (inline `#[cfg(test)]` module): test that `create_pubsub_client` with `PUBSUB_EMULATOR_HOST` set creates a client without credentials. Test that missing credentials returns a descriptive error.

**Checkpoint**: rsudp-rust can authenticate with Pub/Sub (or emulator). Input mode branching in main.rs ready for publisher/subscriber.

---

## Phase 4: User Story 1 - Publish Seismic Data to Pub/Sub (Priority: P1) 🎯 MVP

**Goal**: Raw seismic data is aggregated per 0.5-second window across all channels and published to Pub/Sub with deterministic dedup keys and ordering keys

**Independent Test**: Start rsudp-rust with Pub/Sub enabled + emulator, stream data via streamer, verify messages appear in emulator topic with correct protobuf payload

### Implementation for User Story 1

- [x] T015 [US1] Implement `PubsubPublisher` struct in `rsudp-rust/src/pubsub/publisher.rs` with: `new(publisher: Publisher, station: String, batch_interval_ms: u64)`, internal `HashMap<String, ChannelData>` buffer, `window_start: Option<i64>`, and `sample_rate: f64` tracking
- [x] T016 [US1] Implement `PubsubPublisher::buffer_segment(&mut self, channel: &str, samples: &[i32], start_time_ms: i64, sample_rate: f64)` in `rsudp-rust/src/pubsub/publisher.rs`: accumulate raw integer samples (pre-deconvolution) into per-channel buffers; set `window_start` on first segment of window
- [x] T017 [US1] Implement `PubsubPublisher::flush(&mut self) -> Result<()>` in `rsudp-rust/src/pubsub/publisher.rs`: construct `SeismicBatch` protobuf from buffered channels, generate dedup_key via `dedup::generate_dedup_key()`, create `PubsubMessage` with data (encoded protobuf), attributes (`dedup_key`, `station`, `window_start`), and ordering_key (station). Publish via `self.publisher.publish(msg)`. Clear buffer after success. Log batch details (station, channels, samples count, window).
- [x] T018 [US1] Implement publisher background task: create a `tokio::spawn` task in `rsudp-rust/src/pubsub/publisher.rs` that runs `PubsubPublisher` with an `mpsc::Receiver<SegmentData>` input and a `tokio::time::interval(Duration::from_millis(batch_interval_ms))` flush timer. Define `SegmentData` struct (channel, samples, start_time_ms, sample_rate). Use `tokio::select!` to handle both incoming segments and interval ticks.
- [x] T019 [US1] Integrate publisher into pipeline: in `rsudp-rust/src/pipeline.rs`, add optional `mpsc::Sender<SegmentData>` parameter to `run_pipeline()`. Inside the `for segment in segments` loop, send raw segment data (pre-deconvolution `segment.samples` as `Vec<i32>`) to the publisher channel when present.
- [x] T020 [US1] Wire publisher in `rsudp-rust/src/main.rs`: when `settings.pubsub.enabled && settings.pubsub.input_mode == "udp"`, create topic publisher from client, create `mpsc::channel` for segment data, spawn the publisher background task, pass the sender to `run_pipeline()`.

### Tests for User Story 1

- [x] T021 [P] [US1] Unit test for dedup key generation in `rsudp-rust/src/pubsub/dedup.rs` (`#[cfg(test)]` module): verify `generate_dedup_key("AM.R6E01", 1732525283730)` produces `AM.R6E01:2025-11-25T09:01:23.500Z` (floored to 500ms). Verify same key for timestamps within the same 500ms window. Verify different keys for timestamps in adjacent windows.
- [x] T022 [P] [US1] Unit test for `DedupChecker` in `rsudp-rust/src/pubsub/dedup.rs`: verify `check_and_insert` returns true for new keys, false for duplicates. Verify LRU eviction when max_entries exceeded.
- [x] T023 [P] [US1] Unit test for publisher batching in `rsudp-rust/src/pubsub/publisher.rs` (`#[cfg(test)]` module): verify `buffer_segment` accumulates multiple channels, verify `flush` produces a `SeismicBatch` with correct fields (station, window times, sample_rate, channels with raw i32 samples).

**Checkpoint**: Publisher is functional — data flows from UDP → pipeline → publisher → Pub/Sub topic. Dedup keys are deterministic. Messages contain raw integer samples with correct metadata.

---

## Phase 5: User Story 2 - Subscribe as Alternative Pipeline Input (Priority: P2)

**Goal**: rsudp-rust can receive seismic data from a Pub/Sub subscription instead of UDP, feeding it into the same processing pipeline

**Independent Test**: Publish known test data to emulator topic, start rsudp-rust in subscriber mode (`input_mode = "pubsub"`), verify pipeline receives data (waveform buffers populated)

### Implementation for User Story 2

- [x] T024 [US2] Implement `PubsubSubscriber` struct in `rsudp-rust/src/pubsub/subscriber.rs` with: `new(subscription: Subscription, pipe_tx: mpsc::Sender<Vec<u8>>, station: String)`, internal `DedupChecker`
- [x] T025 [US2] Implement `PubsubSubscriber::run(&self, cancel: CancellationToken)` in `rsudp-rust/src/pubsub/subscriber.rs`: call `subscription.receive()` with callback that (1) extracts `dedup_key` from message attributes, (2) checks dedup via `DedupChecker`, (3) if new: decodes `SeismicBatch` from message data via `prost::Message::decode`, (4) for each `ChannelData` in batch: reconstructs rsudp-compatible JSON packet bytes (matching the format from `parse_any()`), (5) sends each reconstructed packet via `pipe_tx.send()`, (6) acks the message
- [x] T026 [US2] Implement packet reconstruction helper in `rsudp-rust/src/pubsub/subscriber.rs`: `fn reconstruct_packet(station: &str, channel: &ChannelData, sample_rate: f64) -> Vec<u8>` that creates a JSON packet matching the format expected by `parse_any()` in `rsudp-rust/src/parser/mod.rs`, using raw integer samples from protobuf
- [x] T027 [US2] Wire subscriber in `rsudp-rust/src/main.rs`: when `settings.pubsub.enabled && settings.pubsub.input_mode == "pubsub"`, create subscription from client, create `PubsubSubscriber`, spawn `subscriber.run()` task. The subscriber sends directly to `pipe_tx`, replacing the UDP receiver → recv_rx → pipe_tx bridge.

### Tests for User Story 2

- [x] T028 [P] [US2] Unit test for packet reconstruction in `rsudp-rust/src/pubsub/subscriber.rs` (`#[cfg(test)]` module): verify reconstructed packet bytes can be parsed by `parse_any()` and produce a valid segment with correct channel, samples, timestamps, and sample_rate
- [x] T029 [P] [US2] Unit test for subscriber dedup in `rsudp-rust/src/pubsub/subscriber.rs`: verify that processing the same dedup_key twice results in only one `pipe_tx.send()` call

**Checkpoint**: Subscriber mode is functional — data flows from Pub/Sub subscription → subscriber → pipeline. Dedup prevents duplicate processing. UDP listener is not started in subscriber mode.

---

## Phase 6: User Story 4 - Emulator-Based Testing (Priority: P2)

**Goal**: Integration tests run against the Pub/Sub emulator in Docker, covering the full publish-subscribe-deduplicate cycle

**Independent Test**: Start emulator in Docker, set `PUBSUB_EMULATOR_HOST`, run `cargo test --test pubsub_integration`

### Implementation for User Story 4

- [x] T030 [US4] Create `rsudp-rust/tests/pubsub_integration.rs` with test setup: check `PUBSUB_EMULATOR_HOST` env var (skip if not set), create client, create topic and subscription programmatically via admin API
- [x] T031 [US4] Integration test: publish a `SeismicBatch` with known sample data to emulator topic, subscribe and receive the message, verify decoded protobuf matches published data (station, channels, samples, timestamps, sample_rate)
- [x] T032 [US4] Integration test: publish two messages with the same `dedup_key` attribute, verify subscriber-side `DedupChecker` processes only the first
- [x] T033 [US4] Integration test: publish multiple batches with ordering key, verify subscriber receives them in publish order
- [x] T034 [P] [US4] Add Docker Compose service for Pub/Sub emulator: create or update `docker-compose.test.yml` at repo root with `pubsub-emulator` service using `gcr.io/google.com/cloudsdktool/google-cloud-cli:emulators` image, port 8085, and `PUBSUB_EMULATOR_HOST` environment variable for the test runner

**Checkpoint**: Integration tests pass against the emulator, verifying publish, subscribe, dedup, and ordering.

---

## Phase 7: User Story 5 - E2E Test: Full Pipeline via Pub/Sub (Priority: P2)

**Goal**: End-to-end test validates the complete round-trip: UDP streamer → publisher → Pub/Sub emulator → subscriber → pipeline

**Independent Test**: Start emulator + two rsudp-rust processes (publisher + subscriber) + streamer, verify data integrity at subscriber end

### Implementation for User Story 5

- [x] T035 [US5] Create `rsudp-rust/tests/pubsub_e2e.rs` with test harness: start Pub/Sub emulator (or require `PUBSUB_EMULATOR_HOST`), create topic + subscription, generate temp config files for publisher instance (UDP input + Pub/Sub output) and subscriber instance (Pub/Sub input), find available ports for UDP and WebUI
- [x] T036 [US5] Implement E2E test flow in `rsudp-rust/tests/pubsub_e2e.rs`: (1) start publisher rsudp-rust process with publisher config, (2) start subscriber rsudp-rust process with subscriber config, (3) start streamer sending MiniSEED data via UDP to publisher port, (4) wait for data to flow through the full pipeline (~5-10 seconds), (5) verify subscriber received data (check logs or waveform buffer state)
- [x] T037 [US5] Add E2E verification in `rsudp-rust/tests/pubsub_e2e.rs`: verify raw sample values match between streamer input and subscriber output, verify no data windows are lost or duplicated over a 30-second test window by comparing dedup keys, verify channel metadata (station, channel names, sample_rate) is preserved

**Checkpoint**: E2E test passes — full round-trip validated with zero data loss or duplication.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T038 [P] Add structured logging for all Pub/Sub operations in `rsudp-rust/src/pubsub/publisher.rs` and `rsudp-rust/src/pubsub/subscriber.rs`: use `tracing::info!` for successful operations, `tracing::warn!` for retries, `tracing::error!` for failures
- [x] T039 [P] Add retry with exponential backoff for publish failures in `rsudp-rust/src/pubsub/publisher.rs`: implement configurable retry (max 3 attempts, 1s/2s/4s backoff) without blocking the main pipeline. Buffer data during retries.
- [x] T040 [P] Add message size validation in `rsudp-rust/src/pubsub/publisher.rs`: before publishing, check encoded message size. Log warning if approaching 10MB limit (should never happen for 0.5s batches, but defensive check).
- [x] T041 Run `cargo clippy` and fix all warnings in `rsudp-rust/src/pubsub/` modules
- [x] T042 Run `cargo build --release` and verify no compilation errors
- [x] T043 Run full test suite `cargo test` and verify all existing + new tests pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US3 Auth (Phase 3)**: Depends on Foundational — BLOCKS US1 and US2 (client needed)
- **US1 Publish (Phase 4)**: Depends on US3 (needs authenticated client)
- **US2 Subscribe (Phase 5)**: Depends on US3 (needs authenticated client). Independent of US1 for implementation, but integration testing benefits from US1 being done.
- **US4 Integration Tests (Phase 6)**: Depends on US1 and US2 (need publisher + subscriber implemented)
- **US5 E2E Test (Phase 7)**: Depends on US1, US2, and US4 (need full pipeline + emulator setup)
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

```
Phase 1 (Setup)
    ↓
Phase 2 (Foundational)
    ↓
Phase 3 (US3: Auth) ← BLOCKS all Pub/Sub operations
    ↓
  ┌─────────────┐
  ↓             ↓
Phase 4       Phase 5
(US1: Pub)    (US2: Sub)  ← Can run in parallel after US3
  ↓             ↓
  └──────┬──────┘
         ↓
Phase 6 (US4: Integration Tests)
         ↓
Phase 7 (US5: E2E Test)
         ↓
Phase 8 (Polish)
```

### Within Each User Story

- Models/types before services
- Core logic before integration (pipeline/main.rs wiring)
- Unit tests can run in parallel with each other [P]
- Story complete before moving to next priority

### Parallel Opportunities

- T002 and T003 can run in parallel (proto copy and build.rs, different files)
- T007 and T008 can run in parallel (proto.rs and dedup.rs, different files)
- T021, T022, T023 can all run in parallel (different test modules)
- T028 and T029 can run in parallel (different test focuses)
- Phase 4 (US1) and Phase 5 (US2) can run in parallel after US3 completes
- T038, T039, T040 can all run in parallel (different concerns in different functions)

---

## Parallel Example: User Story 1

```bash
# After foundational phase + US3 auth complete:

# Launch unit tests in parallel:
Task T021: "Unit test for dedup key generation in rsudp-rust/src/pubsub/dedup.rs"
Task T022: "Unit test for DedupChecker in rsudp-rust/src/pubsub/dedup.rs"
Task T023: "Unit test for publisher batching in rsudp-rust/src/pubsub/publisher.rs"

# Implementation tasks are sequential (T015 → T016 → T017 → T018 → T019 → T020)
# because each builds on the previous
```

## Parallel Example: US1 + US2 After Auth

```bash
# After Phase 3 (US3: Auth) completes:

# Developer A works on Phase 4 (US1: Publisher):
Task T015-T020: Publisher implementation + T021-T023: Publisher tests

# Developer B works on Phase 5 (US2: Subscriber) in parallel:
Task T024-T027: Subscriber implementation + T028-T029: Subscriber tests
```

---

## Implementation Strategy

### MVP First (User Stories 3 + 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: US3 Auth (CRITICAL — blocks publisher and subscriber)
4. Complete Phase 4: US1 Publish
5. **STOP and VALIDATE**: Test publishing with emulator manually
6. Deploy/demo if ready — publisher is the core capability

### Incremental Delivery

1. Complete Setup + Foundational + Auth → Client connectivity ready
2. Add US1 Publisher → Test with emulator → Deploy (MVP!)
3. Add US2 Subscriber → Test with emulator → Deploy (remote monitoring enabled)
4. Add US4 Integration Tests → CI-ready automated verification
5. Add US5 E2E Test → Full round-trip confidence
6. Polish → Production-ready

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational + Auth together
2. Once Auth (Phase 3) is done:
   - Developer A: US1 Publisher (Phase 4)
   - Developer B: US2 Subscriber (Phase 5)
3. Once both complete:
   - Developer A: US4 Integration Tests (Phase 6)
   - Developer B: US5 E2E Test (Phase 7)
4. Both complete Polish (Phase 8)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- The subscriber's packet reconstruction (T026) must match the format expected by `parse_any()` in `rsudp-rust/src/parser/mod.rs` — study existing JSON packet format carefully
- The publisher must send raw integer samples (pre-deconvolution) — use `segment.samples` directly, not deconvolved values
- `google-cloud-pubsub` auto-detects `PUBSUB_EMULATOR_HOST` — no special code needed for emulator vs. production switching
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
