# Tasks: STA/LTA Alert System

**Input**: Design documents from `/specs/006-sta-lta-alert/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

**Tests**: Included as per "Accuracy" success criterion in spec.md.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Verify project structure in `rsudp-rust/` and ensure `tokio`, `chrono`, and `thiserror` are correctly configured in `rsudp-rust/Cargo.toml`
- [x] T002 Create integration test directory at `rsudp-rust/tests/` if not present
- [x] T003 [P] Ensure `csv` dev-dependency is available for reference data testing in `rsudp-rust/Cargo.toml`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 [P] Define `FilterType` and `FilterConfig` structs in `rsudp-rust/src/trigger.rs`
- [x] T005 [P] Define `AlertEventType` and `AlertEvent` structs in `rsudp-rust/src/trigger.rs`
- [x] T006 [P] Define `AlertConfig` and `AlertState` enum in `rsudp-rust/src/trigger.rs`
- [x] T007 Define `AlertManager` struct with `tokio::sync::mpsc::Sender<AlertEvent>` in `rsudp-rust/src/trigger.rs`
- [x] T008 [P] Implement `AlertManager::new` in `rsudp-rust/src/trigger.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Seismic Event Detection (Priority: P1) 🎯 MVP

**Goal**: Implement real-time STA/LTA calculation, warm-up logic, and alarm triggering.

**Independent Test**: Provide a 100Hz sine wave for warm-up, then a high-amplitude pulse. Verify an "ALARM" event is received on the channel.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T009 [US1] Create unit test for warm-up state transition in `rsudp-rust/src/trigger.rs`
- [x] T010 [US1] Create unit test for threshold exceedance alarm emission in `rsudp-rust/src/trigger.rs`

### Implementation for User Story 1

- [x] T011 [US1] Implement `AlertManager::process_sample` skeleton with timestamp tracking in `rsudp-rust/src/trigger.rs`
- [x] T012 [US1] Integrate existing `RecursiveStaLta` (from Feature 003) into `AlertManager` in `rsudp-rust/src/trigger.rs`
- [x] T013 [US1] Implement `WarmingUp` to `Monitoring` state transition after LTA window duration in `rsudp-rust/src/trigger.rs`
- [x] T014 [US1] Implement "ALARM" event emission when STA/LTA ratio > threshold in `rsudp-rust/src/trigger.rs`

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Automatic System Reset (Priority: P2)

**Goal**: Implement reset logic when ratio falls below threshold and report max ratio during event.

**Independent Test**: Trigger an alarm, then reduce signal to background level. Verify a "RESET" event is received with the correct `max_ratio`.

### Tests for User Story 2

- [x] T015 [US2] Create unit test for reset threshold transition in `rsudp-rust/src/trigger.rs`
- [x] T016 [US2] Create unit test for `max_ratio` tracking and reporting in `rsudp-rust/src/trigger.rs`

### Implementation for User Story 2

- [x] T017 [US2] Implement `max_ratio` tracking in `AlertManager` during `Alarm` state in `rsudp-rust/src/trigger.rs`
- [x] T018 [US2] Implement state transition from `Alarm` to `Monitoring` when ratio < `reset_threshold` in `rsudp-rust/src/trigger.rs`
- [x] T019 [US2] Implement "RESET" event emission with `max_ratio` and end timestamp in `rsudp-rust/src/trigger.rs`

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Noise Reduction via Filtering (Priority: P3)

**Goal**: Implement digital filtering (Butterworth) before STA/LTA calculation.

**Independent Test**: Send a mixed signal (0.5Hz + 10Hz). Apply 1-20Hz bandpass. Verify trigger occurs on 10Hz signal.

### Tests for User Story 3

- [x] T020 [P] [US3] Create unit tests for `Biquad` filter sample processing in `rsudp-rust/src/trigger.rs`
- [x] T021 [US3] Create unit test for Butterworth coefficient generation in `rsudp-rust/src/trigger.rs`

### Implementation for User Story 3

- [x] T022 [P] [US3] Implement `Biquad` struct and Direct Form I processing in `rsudp-rust/src/trigger.rs`
- [x] T023 [US3] Implement Butterworth filter design logic (coefficients for Bandpass/Lowpass/Highpass) in `rsudp-rust/src/trigger.rs`
- [x] T024 [US3] Integrate `Biquad` filter into `AlertManager::process_sample` pipeline in `rsudp-rust/src/trigger.rs`

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T025 [P] Implement data gap detection logic (resetting state on time discontinuity) in `rsudp-rust/src/trigger.rs`
- [x] T026 [P] Implement out-of-order sample discarding in `rsudp-rust/src/trigger.rs`
- [x] T027 [P] Create integration test `rsudp-rust/tests/integration_alert.rs` using real seismic data samples
- [x] T028 [P] Add detailed logging for state transitions and threshold events using `tracing` in `rsudp-rust/src/trigger.rs`
- [x] T029 Run `cargo test` and verify cross-validation with `rsudp-rust/tests/scripts/verify_stalta.py`
- [x] T030 Validate success criteria (latency < 200ms, CPU < 5%) on sample data

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - US1 (Detection) is the priority MVP.
  - US2 (Reset) depends on US1 for triggering.
  - US3 (Filtering) can be developed in parallel with US1/US2 but integration depends on `AlertManager` structure.
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: No dependencies on other stories.
- **User Story 2 (P2)**: Integrates with US1 (Alarm state).
- **User Story 3 (P3)**: Pre-processing step, can be tested in isolation.

### Parallel Opportunities

- All Setup tasks marked [P]
- Foundational tasks T004, T005, T006
- US3 implementation (T022) can start in parallel with US1/US2.
- Polish tasks T025, T026, T027, T028

---

## Parallel Example: User Story 3

```bash
# Implement core filter logic while others work on US1/US2:
Task: "Implement Biquad struct and Direct Form I processing in rsudp-rust/src/trigger.rs"
Task: "Create unit tests for Biquad filter sample processing in rsudp-rust/src/trigger.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 & 2.
2. Complete Phase 3 (US1).
3. **STOP and VALIDATE**: Ensure detection works and alarms are emitted correctly.

### Incremental Delivery

1. Foundation ready.
2. Add US1 (Detection) -> MVP ready.
3. Add US2 (Reset) -> Full event lifecycle.
4. Add US3 (Filtering) -> Robust detection in noisy environments.
5. Apply Polish (Gap handling, detailed logging).

---

## Notes

- [P] tasks = different files or internal logic segments without immediate data dependencies.
- [Story] labels map to US1, US2, US3 from `spec.md`.
- Success criteria validation (T030) is critical for matching `rsudp` performance goals.
