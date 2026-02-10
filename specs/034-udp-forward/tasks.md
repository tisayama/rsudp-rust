# Tasks: UDP Data Forwarding

**Input**: Design documents from `/specs/034-udp-forward/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included as explicitly requested in the feature specification (FR-009, User Story 4).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `rsudp-rust/src/`, `rsudp-rust/tests/` at repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Register forward module and create type definitions

- [x] T001 Register `pub mod forward;` in rsudp-rust/src/lib.rs
- [x] T002 Create rsudp-rust/src/forward.rs with type definitions: `ForwardMsg` enum (Data/Alarm variants), `ForwardError` enum (ConfigMismatch/SocketBind/AddressResolve), `ForwardStats` struct (packets_sent, packets_dropped, send_errors, last_send_at), and `ForwardManager` struct skeleton with private fields (destinations, settings, channels) per contracts/forward-module.md and data-model.md

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core ForwardManager initialization and async task infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 Implement `ForwardManager::new(settings: &ForwardSettings) -> Result<Self, ForwardError>` with: address/port length validation (FR-006), destination address resolution to `SocketAddr`, UDP socket binding per destination (`tokio::net::UdpSocket`), bounded mpsc channel creation (capacity 32) per destination per research.md Decision 6, in rsudp-rust/src/forward.rs
- [x] T004 Implement per-destination async forwarding task loop: `tokio::spawn` one task per destination that receives `ForwardMsg` from its mpsc receiver, calls `UdpSocket::send_to()` for each message, and handles send errors gracefully (log warning, do not panic) per research.md Decision 2, in rsudp-rust/src/forward.rs
- [x] T005 Add `forward_manager: Option<Arc<ForwardManager>>` parameter to `run_pipeline()` function signature in rsudp-rust/src/pipeline.rs (preserve all existing parameters, add as final parameter)
- [x] T006 Initialize `ForwardManager` in rsudp-rust/src/main.rs: conditional on `settings.forward.enabled`, call `ForwardManager::new()`, wrap in `Arc`, handle `ForwardError` with `error!()` log and `process::exit(1)`, pass `Option<Arc<ForwardManager>>` to `run_pipeline()` per contracts/forward-module.md

**Checkpoint**: Foundation ready - ForwardManager initializes, sockets bind, tasks spawn, pipeline accepts forward_manager parameter

---

## Phase 3: User Story 1 - Forward Seismic Data to Remote Receivers (Priority: P1) MVP

**Goal**: Raw seismic data UDP packets are forwarded to all configured destinations with channel filtering

**Independent Test**: Configure one forward destination, send sample seismic data, verify the receiver gets identical raw bytes

### Implementation for User Story 1

- [x] T007 [US1] Implement `should_forward_channel(channel: &str, filters: &[String]) -> bool` helper: case-insensitive suffix matching (e.g., filter "HZ" matches "EHZ"), "all" wildcard matches everything, empty/unmatched filters fall back to all channels with `warn!()` log (FR-011) per research.md Decision 4, in rsudp-rust/src/forward.rs
- [x] T008 [US1] Implement `forward_data(&self, channel: &str, raw_packet: &[u8])` method: check `fwd_data` flag, call `should_forward_channel()`, `try_send(ForwardMsg::Data(raw_packet.to_vec()))` to each destination's mpsc sender, increment `dropped` counter on `TrySendError::Full` per contracts/forward-module.md, in rsudp-rust/src/forward.rs
- [x] T009 [US1] Integrate `forward_data()` calls into `run_pipeline()` in rsudp-rust/src/pipeline.rs: after `parse_any()` produces `Vec<TraceSegment>`, for each segment call `forward_manager.forward_data(&segment.channel, &raw_bytes)` where `raw_bytes` is the original `Vec<u8>` received from the channel per research.md Decision 1
- [x] T010 [US1] Implement `shutdown(&self)` method: drop all mpsc senders to signal task termination, log shutdown event per FR-010, in rsudp-rust/src/forward.rs

### Tests for User Story 1

- [x] T011 [P] [US1] Unit test: single destination data forwarding — bind local UDP listener on random port, create ForwardManager with one destination pointing to listener, call `forward_data()`, assert listener receives identical bytes, in rsudp-rust/tests/test_forward.rs
- [x] T012 [P] [US1] Unit test: multi-destination data forwarding — bind two local UDP listeners, create ForwardManager with two destinations, call `forward_data()`, assert both listeners receive identical bytes, in rsudp-rust/tests/test_forward.rs
- [x] T013 [P] [US1] Unit test: forwarding disabled (enabled=false) — verify no ForwardManager is created and no packets are sent, in rsudp-rust/tests/test_forward.rs

**Checkpoint**: Raw data forwarding works for single and multiple destinations. Channel matching active with "all" default. This is the MVP.

---

## Phase 4: User Story 2 - Filter Forwarded Data by Channel and Message Type (Priority: P2)

**Goal**: Operators can control which channels and message types (data vs. alarms) are forwarded

**Independent Test**: Configure channel filter and fwd_data/fwd_alarms flags, send mixed data, verify only matching packets are forwarded

### Implementation for User Story 2

- [x] T014 [US2] Implement `forward_alarm(&self, message: &str)` method: check `fwd_alarms` flag, format as UTF-8 bytes, `try_send(ForwardMsg::Alarm(message.to_string()))` to each destination per contracts/forward-module.md wire format, in rsudp-rust/src/forward.rs
- [x] T015 [US2] Integrate `forward_alarm()` calls on `AlertEventType::Trigger` and `AlertEventType::Reset` in rsudp-rust/src/pipeline.rs: format as `"ALARM {channel} {ISO-8601 timestamp}"` and `"RESET {channel} {ISO-8601 timestamp}"` per research.md Decision 3

### Tests for User Story 2

- [x] T016 [P] [US2] Unit test: channel filtering — configure `channels = ["EHZ"]`, send data for EHZ, EHN, EHE channels, assert only EHZ data is received by listener, in rsudp-rust/tests/test_forward.rs
- [x] T017 [P] [US2] Unit test: fwd_data=false suppresses data forwarding — configure fwd_data=false, send data, assert listener receives nothing, in rsudp-rust/tests/test_forward.rs
- [x] T018 [P] [US2] Unit test: fwd_alarms flag controls alarm forwarding — configure fwd_alarms=true, call forward_alarm(), assert listener receives ALARM/RESET string; then fwd_alarms=false, assert no alarm received, in rsudp-rust/tests/test_forward.rs

**Checkpoint**: Channel filtering and alarm/data toggle work correctly. Operators can precisely control what is forwarded.

---

## Phase 5: User Story 3 - Verify Forwarding via Runtime Log Monitoring (Priority: P2)

**Goal**: System periodically logs forwarding statistics for operational verification

**Independent Test**: Start with forwarding enabled, wait >60 seconds, verify log contains per-destination stats with packet counts

### Implementation for User Story 3

- [x] T019 [US3] Add `ForwardStats` counter updates to forwarding task loop: increment `packets_sent` on successful `send_to()`, increment `send_errors` on error, update `last_send_at` on success, in rsudp-rust/src/forward.rs
- [x] T020 [US3] Implement periodic stats logging: add `tokio::time::interval(Duration::from_secs(60))` to forwarding task, log per-destination summary `"Forward #N (addr:port): sent=X, dropped=Y, errors=Z"` at INFO level, log delta since last report per data-model.md ForwardStats, in rsudp-rust/src/forward.rs
- [x] T021 [US3] Add startup confirmation log in `ForwardManager::new()`: log `"Forward: N destinations configured [addr1:port1, addr2:port2]"` and `"Forward: channels=[...], fwd_data=X, fwd_alarms=Y"` at INFO level per quickstart.md verification section, in rsudp-rust/src/forward.rs

**Checkpoint**: Operators can verify forwarding status by reading log output. Startup message confirms configuration; periodic stats confirm ongoing operation.

---

## Phase 6: User Story 4 - Automated End-to-End Forwarding Test (Priority: P3)

**Goal**: Automated tests verify the entire forwarding pipeline works end-to-end

**Independent Test**: Run `cargo test test_forward_e2e` — tests start local listeners, send data through forwarding, and assert correctness

### Tests for User Story 4

- [x] T022 [US4] E2E integration test: bind local UDP listener on random port, create `ForwardSettings` with destination pointing to listener, create `ForwardManager`, send 100 sample seismic data packets via `forward_data()`, assert listener received all 100 packets with correct content, verify graceful shutdown, in rsudp-rust/tests/test_forward.rs
- [x] T023 [US4] E2E integration test with filtering: configure `channels = ["EHZ"]` and `fwd_alarms = true`, send mixed data (EHZ + EHN packets) and alarm messages, assert listener received only EHZ data packets and ALARM/RESET messages, in rsudp-rust/tests/test_forward.rs

**Checkpoint**: All E2E tests pass. Forwarding feature is regression-proof.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final quality checks across all user stories

- [x] T024 Run `cargo clippy` on rsudp-rust/ and fix all warnings
- [x] T025 Run quickstart.md validation: build project, configure [forward] in rsudp.toml, verify startup log and periodic stats output match expected format

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational — delivers MVP
- **US2 (Phase 4)**: Depends on Foundational + US1 (forward_alarm uses same infrastructure as forward_data)
- **US3 (Phase 5)**: Depends on Foundational (stats are added to existing forwarding task loop)
- **US4 (Phase 6)**: Depends on US1 + US2 (tests verify complete forwarding behavior)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Depends on Phase 2 only — **independently testable as MVP**
- **US2 (P2)**: Depends on US1 (alarm forwarding uses same task infrastructure) — independently testable after US1
- **US3 (P2)**: Depends on Phase 2 only (adds stats to task loop) — can proceed in parallel with US2
- **US4 (P3)**: Depends on US1 + US2 (E2E tests cover full behavior) — must be last user story

### Within Each User Story

- Implementation tasks before test tasks (tests need working code to verify)
- Core logic before integration (e.g., forward_data before pipeline integration)
- Story complete before moving to next priority

### Parallel Opportunities

- T011, T012, T013 (US1 tests) can run in parallel — different test functions, same file
- T016, T017, T018 (US2 tests) can run in parallel — different test functions, same file
- US2 and US3 can proceed in parallel after US1 completes (different concerns, minimal file overlap in forward.rs)

---

## Parallel Example: User Story 1

```bash
# After T007-T010 implementation completes, launch all US1 tests together:
Task: "Unit test: single destination forwarding in rsudp-rust/tests/test_forward.rs"
Task: "Unit test: multi-destination forwarding in rsudp-rust/tests/test_forward.rs"
Task: "Unit test: forwarding disabled in rsudp-rust/tests/test_forward.rs"
```

## Parallel Example: User Story 2

```bash
# After T014-T015 implementation completes, launch all US2 tests together:
Task: "Unit test: channel filtering in rsudp-rust/tests/test_forward.rs"
Task: "Unit test: fwd_data=false in rsudp-rust/tests/test_forward.rs"
Task: "Unit test: fwd_alarms flag in rsudp-rust/tests/test_forward.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational (T003-T006)
3. Complete Phase 3: User Story 1 (T007-T013)
4. **STOP and VALIDATE**: Test US1 independently — data forwarding works
5. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Infrastructure ready
2. Add US1 → Raw data forwarding works → **MVP!**
3. Add US2 → Channel/alarm filtering works → Production-ready
4. Add US3 → Operational monitoring works → Observable
5. Add US4 → E2E tests pass → Regression-proof
6. Polish → Clean, validated, production-quality

---

## Notes

- [P] tasks = different files or test functions, no dependencies
- [Story] label maps task to specific user story for traceability
- All new code goes in rsudp-rust/src/forward.rs (single new module)
- All new tests go in rsudp-rust/tests/test_forward.rs (single new test file)
- Existing file modifications limited to: pipeline.rs, main.rs, lib.rs
- No new Cargo dependencies required — uses existing tokio, tracing, serde
- Commit after each phase checkpoint
