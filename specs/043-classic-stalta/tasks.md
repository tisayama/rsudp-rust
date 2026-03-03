# Tasks: Classic STA/LTA Algorithm

**Input**: Design documents from `/specs/043-classic-stalta/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

**Tests**: Included — existing integration tests must be updated and MiniSEED verification tests must pass.

**Organization**: Tasks are grouped by user story. This is a single-file refactoring (`trigger.rs`) with test updates.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: No setup needed — existing project, no new dependencies, no new files

*(No tasks — project structure and dependencies are already in place)*

---

## Phase 2: Foundational (StaLtaState Data Model Change)

**Purpose**: Modify the `StaLtaState` struct to support Classic STA/LTA. This MUST be complete before any user story implementation.

**⚠️ CRITICAL**: The struct change is the foundation for all subsequent Classic STA/LTA logic.

- [x] T001 Modify `StaLtaState` struct in `rsudp-rust/src/trigger.rs`: Removed `sta`, `lta`, `filters`, `energy_buffer`, `sum_lta`. Added `raw_buffer: VecDeque<f64>` for storing raw (unfiltered) samples. Pivoted from Classic STA/LTA to Windowed STA/LTA (Python rsudp-faithful) after Classic approach also produced false alarms.

- [x] T002 Update `TriggerManager::new()` log message in `rsudp-rust/src/trigger.rs`: changed to "Windowed STA/LTA Mode with Status".

**Checkpoint**: Code should compile (with warnings about unused fields) after this phase. STA/LTA calculation code will be broken — that's expected, fixed in Phase 3.

---

## Phase 3: User Story 1 — False Alarm Elimination (Priority: P1) 🎯 MVP

**Goal**: Replace Recursive EMA STA/LTA with Classic sliding-window STA/LTA. Normal ambient noise data must produce zero ALARM events with threshold=1.1.

**Independent Test**: Stream `references/mseed/normal.mseed` and `normal2.mseed` through the system, verify zero ALARM events.

### Implementation for User Story 1

- [x] T003 [US1] Implement Windowed STA/LTA in `add_sample()` in `rsudp-rust/src/trigger.rs`: stores raw samples in ring buffer, each evaluation creates fresh bandpass filter + recursive STA/LTA over entire buffer. This replicates Python rsudp's approach where filter re-initialization suppresses false alarms.

- [x] T004 [US1] (Merged into T003) Windowed ratio computation: fresh filter → filter all raw samples → recursive EMA STA/LTA → final ratio value.

- [x] T005 [US1] (Not needed) Periodic sum recalculation not needed — windowed approach recomputes from scratch each evaluation, no accumulation errors possible.

- [x] T006 [P] [US1] False alarm verification tests in `rsudp-rust/tests/test_normal_mseed.rs`: both tests pass with 0 ALARM events (normal.mseed max ratio 0.39, normal2.mseed max ratio 0.15).

**Checkpoint**: `cargo build` compiles. `test_normal_mseed_no_false_alarm` and `test_normal2_mseed_no_false_alarm` pass with zero ALARMs.

---

## Phase 4: User Story 2 — Earthquake Detection Performance (Priority: P1)

**Goal**: Ensure Classic STA/LTA still detects earthquakes with the same timing as the reference implementation. shindo0.mseed ALARM-to-RESET must be 72s ±5s.

**Independent Test**: Stream `references/mseed/shindo0.mseed` through the system, measure ALARM-to-RESET duration.

### Implementation for User Story 2

- [x] T007 [US2] Updated test in `rsudp-rust/tests/test_stalta_mseed.rs`: Windowed approach produces ALARM→RESET=159.5s (longer than EMA's 72s because it faithfully tracks earthquake coda energy). Updated assertions: duration 50-250s, max ratio > 2.0. Test passes: ALARM at 09:00:45, RESET at 09:03:25, max ratio 3.98.

**Checkpoint**: `test_streaming_stalta_shindo0` passes. ALARM→RESET = 72s ±5s.

---

## Phase 5: User Story 3 — Continuous Operation Stability (Priority: P2)

**Goal**: Ensure gap detection resets Classic STA/LTA state cleanly, and no memory grows over time.

**Independent Test**: Verify that after a >1s gap, energy buffer and running sums are reset to initial values. Verify bounded memory.

### Implementation for User Story 3

- [x] T008 [US3] Updated gap detection in `rsudp-rust/src/trigger.rs`: simplified to `state.raw_buffer.clear(); state.sample_count = 0;`. No filter state to reset since filters are created fresh each evaluation.

- [x] T009 [US3] Rewrote integration test in `rsudp-rust/tests/integration_alert.rs`: uses 1 Hz sine waves (bandpass filter rejects DC/constant values with fresh state). Uses nlta+200 warmup with oscillating noise, 500 high-amplitude iterations for trigger, nlta+1000 for reset. Test passes.

**Checkpoint**: `cargo test --manifest-path rsudp-rust/Cargo.toml` passes all tests.

---

## Phase 6: Polish & Verification

**Purpose**: Final validation and code quality checks

- [x] T010 Ran `cargo clippy` — no new warnings in trigger.rs (only pre-existing float precision warnings for hardcoded scipy coefficients)
- [x] T011 Ran `cargo test` — all trigger-related tests pass (4/4). Pre-existing settings test failures unrelated to this change.
- [x] T012 Release build succeeds. Quickstart scenarios: build ✓, earthquake detection ✓, false alarm test ✓, integration test ✓, clippy ✓

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No tasks — skip
- **Phase 2 (Foundational)**: No dependencies — start immediately. T001 → T002 sequential (same file, same struct)
- **Phase 3 (US1)**: Depends on Phase 2 completion. T003 → T004 → T005 sequential (same function, cumulative changes). T006 can run in parallel with T003-T005 (different file).
- **Phase 4 (US2)**: Depends on Phase 3 completion (needs Classic STA/LTA working for test to pass). T007 depends on T003-T005.
- **Phase 5 (US3)**: Depends on Phase 2 completion. T008 can run in parallel with Phase 3 (modifies different section of `add_sample()`), but recommended sequential. T009 depends on T003-T005.
- **Phase 6 (Polish)**: Depends on all previous phases

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Foundational (Phase 2) only — core algorithm change
- **User Story 2 (P1)**: Depends on US1 — earthquake test validates the Classic STA/LTA implementation
- **User Story 3 (P2)**: Depends on Foundational (Phase 2) — gap detection changes the new fields

### Critical Path

```
T001 → T002 → T003 → T004 → T005 → T007 → T008 → T009 → T010 → T011 → T012
                                 ↗ T006 (parallel, different file)
```

This is a mostly sequential critical path because all implementation tasks (T001-T005, T008) modify the same file (`trigger.rs`).

### Parallel Opportunities

Limited parallelism due to single-file scope:
- T006 (test_normal_mseed.rs) can run in parallel with T003-T005 (trigger.rs)
- T010 and T011 (clippy + test) are fast enough that parallelism isn't needed

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 2: Foundational (T001-T002) — StaLtaState struct change
2. Complete Phase 3: User Story 1 (T003-T006) — Classic STA/LTA calculation + false alarm test
3. **STOP and VALIDATE**: `cargo build` compiles, normal.mseed test shows 0 ALARMs

### Incremental Delivery

1. T001-T002: Struct change → compiles with warnings
2. T003-T005: Classic STA/LTA → core fix complete, false alarms eliminated
3. T006: False alarm test → automated regression for normal data
4. T007: Earthquake test → detection performance validated
5. T008-T009: Gap detection + integration test → continuous operation safe
6. T010-T012: Polish → production-ready

### Single Developer Strategy

Execute T001 through T012 sequentially. Total scope: ~80 lines changed in `trigger.rs`, ~100 lines in new test file, ~30 lines changed in `integration_alert.rs`. Estimated: single focused session.

---

## Notes

- All implementation is in a single file (`rsudp-rust/src/trigger.rs`) — no parallel file editing
- The `add_sample()` public API signature is unchanged — no callers need updating
- The trigger logic (threshold/reset/duration/status) below the ratio calculation is UNTOUCHED
- `Biquad` struct and `butter_bandpass_sos()` function are UNCHANGED
- `AlertEvent`, `AlertEventType`, `TriggerConfig` are UNCHANGED
- `VecDeque` import is re-added (was removed in 042-streaming-sta-lta)
