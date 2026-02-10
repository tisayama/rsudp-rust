# Tasks: RSAM Calculation and UDP Forwarding

**Input**: Design documents from `/specs/035-rsam-calculation/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included as explicitly requested in the feature specification (User Story 4, E2E tests).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `rsudp-rust/src/`, `rsudp-rust/tests/` at repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Register RSAM module and create type definitions

- [ ] T001 Register `pub mod rsam;` in rsudp-rust/src/lib.rs
- [ ] T002 Create rsudp-rust/src/rsam.rs with type definitions: `RsamError` enum (AddressResolve/SocketBind), `RsamResult` struct (station, channel, mean, median, min, max), and `RsamManager` struct skeleton with private fields (settings, socket, dest_addr, buffer, last_calc_time, station, matched_channel, sensitivity, sensitivity_map, warm) per contracts/rsam-module.md and data-model.md

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core RsamManager initialization and pipeline integration that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

- [ ] T003 Implement `RsamManager::new(settings: &RsamSettings, sensitivity_map: HashMap<String, f64>) -> Result<Self, RsamError>` with: destination address resolution to `SocketAddr`, UDP socket binding (`std::net::UdpSocket`), unknown fwformat fallback to LITE with `warn!()` log, in rsudp-rust/src/rsam.rs
- [ ] T004 Implement `RsamResult` formatting methods: `format_lite()` producing `stn:{station}|ch:{channel}|mean:{v}|med:{v}|min:{v}|max:{v}`, `format_json()` producing valid JSON object, `format_csv()` producing comma-separated line, and `format(fwformat: &str)` dispatcher with unknown-format fallback to LITE, in rsudp-rust/src/rsam.rs
- [ ] T005 Add `rsam_manager: Option<RsamManager>` parameter to `run_pipeline()` function signature in rsudp-rust/src/pipeline.rs (preserve all existing parameters, add as final parameter)
- [ ] T006 Initialize `RsamManager` in rsudp-rust/src/main.rs: conditional on `settings.rsam.enabled`, call `RsamManager::new(&settings.rsam, sens_map.clone())`, handle `RsamError` with `error!()` log and `process::exit(1)`, pass `Option<RsamManager>` to `run_pipeline()` per contracts/rsam-module.md

**Checkpoint**: Foundation ready — RsamManager initializes, socket binds, formatting works, pipeline accepts rsam_manager parameter

---

## Phase 3: User Story 1 - Calculate and Forward RSAM Values (Priority: P1) MVP

**Goal**: RSAM statistics (mean, median, min, max of absolute amplitudes) are calculated and sent via UDP at configured intervals

**Independent Test**: Configure RSAM with one destination, send sample data, verify receiver gets correctly formatted RSAM statistics

### Implementation for User Story 1

- [ ] T007 [US1] Implement `process_segment(&mut self, segment: &TraceSegment)` method: check channel suffix match using `forward::should_forward_channel()`, on first match set station/matched_channel/sensitivity from segment metadata and sensitivity_map, append `segment.samples` absolute values to buffer, check if `last_calc_time.elapsed() >= interval`, if so call `calculate()` and send via UDP, reset buffer and timer, in rsudp-rust/src/rsam.rs
- [ ] T008 [US1] Implement `calculate(&self) -> Option<RsamResult>` method: return None if buffer is empty, compute mean (sum/len), median (sort + middle), min, max of buffer values, return `RsamResult` with station and channel, in rsudp-rust/src/rsam.rs
- [ ] T009 [US1] Implement UDP send logic in `process_segment()`: after `calculate()` returns Some, format result using `result.format(&self.settings.fwformat)`, call `socket.send_to(formatted.as_bytes(), dest_addr)`, handle send errors with `warn!()` log (FR-010), in rsudp-rust/src/rsam.rs
- [ ] T010 [US1] Integrate `process_segment()` calls into `run_pipeline()` in rsudp-rust/src/pipeline.rs: after `parse_any()` produces segments, for each segment call `rsam_manager.process_segment(&segment)` per contracts/rsam-module.md

### Tests for User Story 1

- [ ] T011 [P] [US1] Unit test: RSAM calculation correctness — create RsamManager, feed known sample values [10.0, -20.0, 30.0, -40.0, 50.0], call `calculate()`, assert mean=30.0, median=30.0, min=10.0, max=50.0 (absolute values), in rsudp-rust/tests/test_rsam.rs
- [ ] T012 [P] [US1] Unit test: LITE format output — create RsamResult with known values, call `format_lite()`, assert output matches `stn:TEST|ch:EHZ|mean:30|med:30|min:10|max:50` pattern, in rsudp-rust/tests/test_rsam.rs
- [ ] T013 [P] [US1] Unit test: JSON format output — create RsamResult, call `format_json()`, parse result as JSON, assert all fields present and correct, in rsudp-rust/tests/test_rsam.rs
- [ ] T014 [P] [US1] Unit test: CSV format output — create RsamResult, call `format_csv()`, assert comma-separated fields match expected values, in rsudp-rust/tests/test_rsam.rs
- [ ] T015 [P] [US1] Integration test: UDP delivery — bind local UDP listener, create RsamManager pointing to listener, feed segments until interval elapses, assert listener receives correctly formatted RSAM packet, in rsudp-rust/tests/test_rsam.rs

**Checkpoint**: RSAM calculation and UDP forwarding work. All three formats produce correct output. This is the MVP.

---

## Phase 4: User Story 2 - Channel Matching and Filtering (Priority: P2)

**Goal**: RSAM only processes data from the configured channel using suffix matching

**Independent Test**: Configure RSAM with `channel = "HZ"`, send multi-channel data, verify only HZ-suffixed channel is included

### Implementation for User Story 2

- [ ] T016 [US2] Verify and test channel suffix matching in `process_segment()`: reuses `forward::should_forward_channel()` with single-element filter `[self.settings.channel.clone()]`, ensure case-insensitive matching works for RSAM (e.g., "HZ" matches "EHZ", "SHZ"), in rsudp-rust/src/rsam.rs

### Tests for User Story 2

- [ ] T017 [P] [US2] Unit test: channel filtering — create RsamManager with `channel="HZ"`, feed segments for EHZ, EHN, EHE channels, trigger calculation, assert only EHZ data is included in result, in rsudp-rust/tests/test_rsam.rs
- [ ] T018 [P] [US2] Unit test: suffix matching variations — test that channel="HZ" matches both "EHZ" and "SHZ", that channel="ENZ" matches "ENZ" but not "ENE", in rsudp-rust/tests/test_rsam.rs

**Checkpoint**: Channel filtering works correctly. Only matching channel data enters RSAM calculation.

---

## Phase 5: User Story 3 - Verify RSAM Operation via Logs (Priority: P2)

**Goal**: System logs RSAM configuration at startup and periodic results when quiet=false

**Independent Test**: Start with RSAM enabled and `quiet=false`, verify log includes RSAM statistics

### Implementation for User Story 3

- [ ] T019 [US3] Add startup confirmation log in `RsamManager::new()`: log `"RSAM: channel={}, interval={}s, format={}, destination={}:{}, deconvolve={}, units={}"` at INFO level per FR-008, in rsudp-rust/src/rsam.rs
- [ ] T020 [US3] Add periodic RSAM result logging in `process_segment()`: when `quiet=false` and a calculation is performed, log `"RSAM [{}]: mean={:.2}, median={:.2}, min={:.2}, max={:.2}"` at INFO level per FR-009, in rsudp-rust/src/rsam.rs

**Checkpoint**: Operators can verify RSAM status from logs. Startup message confirms configuration; periodic stats (when quiet=false) confirm ongoing operation.

---

## Phase 6: User Story 1 Addendum - Deconvolution Support (Priority: P1)

**Goal**: When deconvolve=true, convert raw counts to physical units using sensitivity map before RSAM calculation

**Independent Test**: Configure RSAM with `deconvolve=true`, known sensitivity, feed known samples, verify RSAM values reflect sensitivity-adjusted physical units

### Implementation for Deconvolution

- [ ] T021 [US1] Implement deconvolution logic in `process_segment()`: when `self.settings.deconvolve` is true and sensitivity is available, divide each sample by sensitivity before taking absolute value; for GRAV mode, additionally divide by 9.81; for CHAN mode, resolve to VEL for EH* channels and ACC for EN* channels; fall back to raw counts with `warn!()` if no sensitivity available (FR-013, FR-014, FR-015), in rsudp-rust/src/rsam.rs

### Tests for Deconvolution

- [ ] T022 [P] [US1] Unit test: deconvolution with known sensitivity — create RsamManager with deconvolve=true, units=VEL, sensitivity_map={"EHZ": 1000.0}, feed samples [1000.0, -2000.0], assert RSAM values are [1.0, 2.0] (divided by sensitivity), in rsudp-rust/tests/test_rsam.rs
- [ ] T023 [P] [US1] Unit test: deconvolution GRAV mode — create RsamManager with units=GRAV, sensitivity_map={"ENZ": 100.0}, feed sample [981.0], assert result ≈ 1.0 (981/100/9.81), in rsudp-rust/tests/test_rsam.rs
- [ ] T024 [P] [US1] Unit test: deconvolution fallback — create RsamManager with deconvolve=true but empty sensitivity_map, feed samples, assert raw counts are used (no division), in rsudp-rust/tests/test_rsam.rs

**Checkpoint**: Deconvolution works for all unit modes (VEL, ACC, DISP, GRAV, CHAN). Fallback to raw counts when sensitivity unavailable.

---

## Phase 7: User Story 4 - Automated End-to-End RSAM Tests (Priority: P3)

**Goal**: Automated tests verify the entire RSAM pipeline works end-to-end

**Independent Test**: Run `cargo test test_rsam_e2e` — tests start local listeners, send data through RSAM, and assert correctness

### Tests for User Story 4

- [ ] T025 [US4] E2E test: bind local UDP listener, create RsamManager with LITE format pointing to listener, feed 200 samples (2-second interval worth at 100Hz) via `process_segment()`, trigger interval, assert listener received one RSAM packet with correct LITE format and accurate mean/median/min/max values, in rsudp-rust/tests/test_rsam.rs
- [ ] T026 [US4] E2E test with JSON format: same as T025 but with fwformat=JSON, verify received packet is valid JSON with correct field values, in rsudp-rust/tests/test_rsam.rs
- [ ] T027 [US4] E2E test with deconvolution and filtering: configure channel="EHZ", deconvolve=true, units=VEL, sensitivity=399000000.0, send mixed EHZ+EHN data, assert only deconvolved EHZ data appears in RSAM result, in rsudp-rust/tests/test_rsam.rs
- [ ] T028 [US4] E2E test with multiple intervals: configure interval=2, send data for 5+ seconds, assert at least 2 RSAM packets received with different values (confirming buffer reset between intervals), in rsudp-rust/tests/test_rsam.rs

**Checkpoint**: All E2E tests pass. RSAM feature is regression-proof.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final quality checks across all user stories

- [ ] T029 Run `cargo clippy` on rsudp-rust/ and fix all warnings
- [ ] T030 Run quickstart.md validation: build project, configure [rsam] in rsudp.toml, verify startup log and RSAM output match expected format

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational — delivers MVP
- **US2 (Phase 4)**: Depends on Foundational + US1 (uses same process_segment method)
- **US3 (Phase 5)**: Depends on Foundational + US1 (adds logging to existing methods)
- **Deconv (Phase 6)**: Depends on US1 (extends process_segment with conversion logic)
- **US4 (Phase 7)**: Depends on US1 + US2 + Deconv (E2E tests cover full behavior)
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Depends on Phase 2 only — **independently testable as MVP**
- **US2 (P2)**: Depends on US1 (channel filtering is part of process_segment) — independently testable after US1
- **US3 (P2)**: Depends on US1 (logging added to existing calculation flow) — can proceed in parallel with US2
- **Deconv**: Depends on US1 (extends sample processing in process_segment)
- **US4 (P3)**: Depends on US1 + US2 + Deconv (E2E tests cover full behavior) — must be last user story

### Within Each User Story

- Implementation tasks before test tasks (tests need working code to verify)
- Core logic before integration (e.g., calculate() before pipeline integration)
- Story complete before moving to next priority

### Parallel Opportunities

- T011, T012, T013, T014, T015 (US1 tests) can run in parallel — different test functions, same file
- T017, T018 (US2 tests) can run in parallel — different test functions, same file
- T022, T023, T024 (Deconv tests) can run in parallel — different test functions, same file
- US2 and US3 can proceed in parallel after US1 completes (different concerns)

---

## Parallel Example: User Story 1

```bash
# After T007-T010 implementation completes, launch all US1 tests together:
Task: "Unit test: RSAM calculation correctness in rsudp-rust/tests/test_rsam.rs"
Task: "Unit test: LITE format output in rsudp-rust/tests/test_rsam.rs"
Task: "Unit test: JSON format output in rsudp-rust/tests/test_rsam.rs"
Task: "Unit test: CSV format output in rsudp-rust/tests/test_rsam.rs"
Task: "Integration test: UDP delivery in rsudp-rust/tests/test_rsam.rs"
```

## Parallel Example: Deconvolution Tests

```bash
# After T021 implementation completes, launch all deconv tests together:
Task: "Unit test: deconvolution with known sensitivity in rsudp-rust/tests/test_rsam.rs"
Task: "Unit test: deconvolution GRAV mode in rsudp-rust/tests/test_rsam.rs"
Task: "Unit test: deconvolution fallback in rsudp-rust/tests/test_rsam.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational (T003-T006)
3. Complete Phase 3: User Story 1 (T007-T015)
4. **STOP and VALIDATE**: Test US1 independently — RSAM calculates and sends via UDP
5. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Infrastructure ready
2. Add US1 → RSAM calculation and UDP sending works → **MVP!**
3. Add US2 → Channel filtering works → Production-ready
4. Add US3 → Operational logging works → Observable
5. Add Deconv → Sensitivity conversion works → Full Python parity
6. Add US4 → E2E tests pass → Regression-proof
7. Polish → Clean, validated, production-quality

---

## Notes

- [P] tasks = different files or test functions, no dependencies
- [Story] label maps task to specific user story for traceability
- All new code goes in rsudp-rust/src/rsam.rs (single new module)
- All new tests go in rsudp-rust/tests/test_rsam.rs (single new test file)
- Existing file modifications limited to: pipeline.rs, main.rs, lib.rs
- No new Cargo dependencies required — uses existing tokio, tracing, serde, std::net
- Reuses `forward::should_forward_channel()` for channel matching
- RsamSettings already exists in settings.rs — no config changes needed
- Commit after each phase checkpoint
